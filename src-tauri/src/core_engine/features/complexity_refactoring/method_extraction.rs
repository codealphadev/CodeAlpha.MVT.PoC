use std::collections::HashMap;

use cached::proc_macro::cached;

use tree_sitter::Node;

use super::{
    complexity_refactoring::Edit, get_node_address, get_slice_inputs_and_outputs,
    get_sub_slice_inputs_and_outputs, is_child_of, update_parsing_metadata_for_node,
    ComplexityRefactoringError, NodeSubSlice, SliceInputsAndOutputs,
};
use crate::core_engine::{
    features::complexity_refactoring::{
        refactor_function, NodeAddress, NodeSlice, ParsingMetadata, SerializedNodeSlice,
    },
    rules::TemporaryFileOnDisk,
    syntax_tree::{
        calculate_cognitive_complexities, is_expression, is_l_expression, Complexities,
        SwiftFunction, SwiftSyntaxTree,
    },
    SwiftLspError, TextPosition, XcodeText,
};
use cached::SizedCache;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tracing::{debug, error};

#[cached(
    type = "SizedCache<String, Option<(SerializedNodeSlice, isize)>>",
    create = "{ SizedCache::with_size(100) }",
    convert = r#"{ String::from(function.props.node.to_sexp()) }"#,
    result = true
)]
pub fn check_for_method_extraction(
    function: &SwiftFunction,
    text_content: &XcodeText,
    syntax_tree: &SwiftSyntaxTree,
) -> Result<Option<(SerializedNodeSlice, isize)>, ComplexityRefactoringError> {
    let node = function.props.node;
    // Build up a list of possible nodes to extract, each with relevant metrics used for comparison

    let node_address = vec![node.id()];
    let mut parsing_metadata = ParsingMetadata::new(node_address.clone());

    let possible_extractions: Vec<NodeSlice> = walk_node(
        node,
        text_content,
        syntax_tree,
        node_address,
        &mut parsing_metadata,
    )?;

    let function_complexity = syntax_tree
        .get_node_metadata(&node)
        .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
        .complexities
        .clone();

    const SCORE_THRESHOLD: f64 = 0.6;

    Ok(get_best_extraction(
        possible_extractions,
        syntax_tree,
        text_content,
        function_complexity.clone(),
        &parsing_metadata,
        SCORE_THRESHOLD,
    )?
    .map(|(slice, remaining_complexity)| (slice.serialize(node), remaining_complexity)))
}

pub async fn get_edits_for_method_extraction(
    start_position: TextPosition,
    range_length: usize,
    set_result_callback: impl FnOnce(Vec<Edit>) -> () + Send + 'static,
    text_content: &XcodeText,
    file_path: Option<String>,
) -> Result<(), ComplexityRefactoringError> {
    debug!("Getting edits for method extraction");
    // Create temporary file
    let tmp_file_key = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    let temp_file = create_temp_file(&text_content, tmp_file_key)?;
    let suggestion = refactor_function(
        &file_path.unwrap(),
        start_position,
        range_length,
        &text_content,
        &temp_file.path.to_string_lossy().to_string(),
    )
    .await
    .map_err(|e| {
        delete_temp_file(&temp_file);
        match e {
            SwiftLspError::RefactoringNotPossible(payload) => {
                ComplexityRefactoringError::LspRejectedRefactoring(payload)
            }
            _ => ComplexityRefactoringError::GenericError(e.into()),
        }
    })?;

    delete_temp_file(&temp_file);
    set_result_callback(suggestion);

    Ok(())
}

fn delete_temp_file(temp_file: &TemporaryFileOnDisk) {
    match TemporaryFileOnDisk::delete(&temp_file) {
        Err(e) => {
            error!(?e, "Could not clean up temp file")
        }
        Ok(_) => (),
    }
}

fn create_temp_file(
    text_content: &XcodeText,
    key: String,
) -> Result<TemporaryFileOnDisk, ComplexityRefactoringError> {
    let file_name = format!("pretzl_{}_method_extraction.swift", key);
    let path_buf = std::env::temp_dir().join(file_name);

    let file = TemporaryFileOnDisk::new(path_buf, text_content.as_string());
    file.write()
        .map_err(|err| ComplexityRefactoringError::GenericError(err.into()))?;

    Ok(file)
}
fn get_best_extraction<'a>(
    candidates: Vec<NodeSlice<'a>>,
    syntax_tree: &'a SwiftSyntaxTree,
    text_content: &'a XcodeText,
    original_complexity: Complexities,
    parsing_metadata: &ParsingMetadata,
    score_threshold: f64,
) -> Result<Option<(NodeSlice<'a>, isize)>, ComplexityRefactoringError> {
    let mut best_possibility = None;
    let mut best_score = 0.0;

    let mut output_remaining_complexity: Option<isize> = None;

    for candidate_slice in candidates {
        let ComplexitiesPrediction {
            removed_complexity,
            new_function_complexity,
        } = get_resulting_complexities(candidate_slice.clone(), syntax_tree, text_content)?;

        let remaining_complexity =
            (original_complexity.clone() - removed_complexity).get_total_complexity();

        let SliceInputsAndOutputs {
            input_names,
            output_names,
        } = get_slice_inputs_and_outputs(&candidate_slice, &parsing_metadata);

        let score = evaluate_suggestion_score(
            input_names.len(),
            output_names.len(),
            original_complexity.get_total_complexity(),
            new_function_complexity.get_total_complexity(),
            remaining_complexity,
        );

        if score > best_score && score > score_threshold {
            best_possibility = Some(candidate_slice);
            best_score = score;
            output_remaining_complexity = Some(remaining_complexity);
        }
    }
    Ok(best_possibility.map(|slice| {
        let slice_kinds = slice.nodes.iter().map(|n| n.kind()).collect::<Vec<&str>>();
        let parent = slice.nodes.first().unwrap().parent().map(|n| n.kind());
        debug!(
            ?slice_kinds,
            ?best_score,
            ?original_complexity,
            ?output_remaining_complexity,
            ?parent,
            "Found refactoring suggestion"
        );
        (
            slice,
            output_remaining_complexity.expect("Remaining complexity should be set"),
        )
    }))
}

fn evaluate_suggestion_score(
    input_count: usize,
    output_count: usize,
    original_complexity: isize,
    new_function_complexity: isize,
    remaining_complexity: isize,
) -> f64 {
    // Should be higher than 1, to incentivise equalizing complexity of the two functions
    let equality_preference_factor = 1.35;
    let input_count_exponent = 1.7;
    original_complexity as f64
        - get_p_norm(
            remaining_complexity as f64,
            new_function_complexity as f64,
            equality_preference_factor,
        )
        - f64::powf(input_count as f64, input_count_exponent) * 0.19
        - output_count as f64 * 0.25
}

fn get_p_norm(x: f64, y: f64, exponent: f64) -> f64 {
    f64::powf(
        f64::powf(x, exponent) + f64::powf(y, exponent),
        1.0 / exponent,
    )
}

fn walk_node<'a>(
    node: Node<'a>,
    text_content: &XcodeText,
    syntax_tree: &'a SwiftSyntaxTree,
    node_address: NodeAddress,
    parsing_metadata: &mut ParsingMetadata,
) -> Result<Vec<NodeSlice<'a>>, ComplexityRefactoringError> {
    let mut possible_extractions: Vec<NodeSlice<'a>> = Vec::new();

    let mut cursor = node.walk();

    update_parsing_metadata_for_node(parsing_metadata, &node, &node_address, &text_content)?;

    for child in node.named_children(&mut cursor) {
        possible_extractions.append(&mut walk_node(
            child,
            text_content,
            syntax_tree,
            get_node_address(&node_address, child.id()),
            parsing_metadata,
        )?);
    }
    if node_children_are_candidates_for_extraction(&node) {
        let node_children = node.children(&mut cursor).collect::<Vec<Node<'a>>>();

        possible_extractions.append(&mut get_valid_slices_for_extraction(
            node_children,
            &node_address,
            &parsing_metadata,
        ));
    }

    if node_is_candidate_for_extraction(&node) {
        let mut parent_address = node_address;
        parent_address.pop();
        possible_extractions.append(&mut get_valid_slices_for_extraction(
            vec![node],
            &parent_address,
            &parsing_metadata,
        ));
    };

    Ok(possible_extractions)
}

fn get_valid_slices_for_extraction<'a, 'b>(
    nodes: Vec<Node<'a>>,
    parent_address: &NodeAddress,
    parsing_metadata: &'b ParsingMetadata,
) -> Vec<NodeSlice<'a>> {
    let mut result: Vec<NodeSlice> = vec![];

    // If range is contained by, or contains, a lambda closure, SourceKit does not correctly determine the input parameters to the function.
    // This would leave the code in a broken state. (We might be able to fix this upstream, or patch in the params ourselves with more resources.)
    if has_parent_of_kind(nodes[0], "lambda_literal") {
        return vec![];
    }

    for (i, start_node) in nodes.iter().enumerate() {
        if !start_node.is_named() {
            // A slice may only consiste of named nodes
            continue;
        }
        if !is_node_valid_for_extraction(start_node, parsing_metadata, parent_address) {
            continue;
        }
        for (j, end_node) in nodes.iter().enumerate().skip(i) {
            if !end_node.is_named() {
                // A slice may only consiste of named nodes
                continue;
            }
            if !is_node_valid_for_extraction(end_node, parsing_metadata, parent_address) {
                break;
            }
            if is_slice_valid_for_extraction(
                NodeSubSlice {
                    nodes: &nodes[i..=j],
                    parent_address: &parent_address.clone(),
                },
                parsing_metadata,
            ) {
                result.push(NodeSlice {
                    nodes: (&nodes[i..=j]).to_vec(),
                    parent_address: parent_address.clone(),
                })
            }
        }
    }
    result
}

fn has_parent_of_kind(node: Node, kind: &str) -> bool {
    let mut curr_node = node.parent();
    while let Some(n) = curr_node {
        if n.kind() == kind {
            return true;
        }
        curr_node = n.parent();
    }
    false
}

fn is_or_has_named_child_of_kind(node: &Node, kind: &str) -> bool {
    if node.kind() == kind {
        return true;
    }
    for child in node.named_children(&mut node.walk()) {
        if is_or_has_named_child_of_kind(&child, kind) {
            return true;
        }
    }
    false
}

fn is_node_valid_for_extraction<'a, 'b>(
    node: &'a Node,
    parsing_metadata: &'b ParsingMetadata,
    parent_address: &NodeAddress,
) -> bool {
    if node.kind() == "guard_statement" {
        return false;
    }

    let node_address = get_node_address(&parent_address, node.id());

    if is_or_has_named_child_of_kind(node, "lambda_literal") {
        return false;
    }
    for continue_or_break in &parsing_metadata.continues_and_breaks {
        if is_child_of(&node_address, &continue_or_break.node_address)
            && !is_child_of(&node_address, &continue_or_break.target_node_address)
        {
            return false;
        }
    }
    true
}

fn is_slice_valid_for_extraction(slice: NodeSubSlice, parsing_metadata: &ParsingMetadata) -> bool {
    let SliceInputsAndOutputs {
        input_names: _,
        output_names,
    } = get_sub_slice_inputs_and_outputs(&slice, parsing_metadata);

    if output_names.len() > 0 {
        // SourceKit cannot extract method if any declaration declared within the slice is referred to after it
        return false;
    }

    return true;
}

fn node_children_are_candidates_for_extraction(node: &Node) -> bool {
    node.kind() == "statements" // Restricting to blocks for now
}

fn node_is_candidate_for_extraction(node: &Node) -> bool {
    is_expression(node.kind())
        && !is_l_expression(node.kind())
        && node.parent().map(|n| is_expression(n.kind())) != Some(true) // Disallow any expressions who are themselves parts of expressions(PartOfExpression)
}
// We need to track which variables were declared within each scope, because global variables should be ignored (and can't be found).
// There can be cases where we have an ERROR node etc., so just return None if no name is found
// TODO: Should we handle this differently? Maybe don't check for method extraction if an error is contained in the node. Then treat this as a real error if the assertion of simple_identifier fails.

struct ComplexitiesPrediction {
    removed_complexity: Complexities,
    new_function_complexity: Complexities,
}

fn get_resulting_complexities(
    method_extraction: NodeSlice,
    syntax_tree: &SwiftSyntaxTree,
    text_content: &XcodeText,
) -> Result<ComplexitiesPrediction, ComplexityRefactoringError> {
    let removed_complexity = method_extraction.nodes.iter().try_fold(
        Complexities::new(),
        |acc, n| -> Result<Complexities, ComplexityRefactoringError> {
            Ok(acc
                + syntax_tree
                    .get_node_metadata(n)
                    .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
                    .complexities
                    .clone())
        },
    )?;

    let mut new_function_complexity = Complexities::new();

    for node in method_extraction.nodes {
        // Start depth at 1, since we assume wrapping nodes in a function_declaration
        new_function_complexity +=
            calculate_cognitive_complexities(&node, &text_content, &mut HashMap::new(), Some(1))
                .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
    }

    return Ok(ComplexitiesPrediction {
        removed_complexity,
        new_function_complexity,
    });
}

#[cfg(test)]
mod tests {
    mod method_extraction {

        use crate::core_engine::{
            features::complexity_refactoring::check_for_method_extraction,
            syntax_tree::{SwiftFunction, SwiftSyntaxTree},
            XcodeText,
        };

        #[test]
        fn makes_correct_suggestion_for_block_of_statements() {
            let text_content = XcodeText::from_str(
                r#"
                public func extractName(input: String) -> String {
                    if input is String {                                // + 1
                        let start = String(input.prefix(1))
                        let end = String(input.suffix(1));
                        var result = start + end;
                        return result;
                    } else if input is Int {                            // + 1
                        let result: Int;
                        if (Int(input) ?? 0 < 1) {                      // + 2 (1 for nesting)
                            result = 0;
                        }
                        var a: Int = 0;
                        var b = 1;
                        for i in 1..<(Int(input) ?? 0) {                // + 2 (1 for nesting)
                            let c = a + b;
                            a = b;
                            b = c;
                        }
                        result = c;
                        result = b;
                        return String(b);
                    } else {                                            // + 1
                        return "undefined";
                    }
                }
            "#,
            );

            let mut swift_syntax_tree = SwiftSyntaxTree::new();
            swift_syntax_tree.parse(&text_content).unwrap();

            let functions =
                SwiftFunction::get_top_level_functions(&swift_syntax_tree, &text_content).unwrap();
            assert_eq!(functions.len(), 1);

            let result =
                check_for_method_extraction(&functions[0], &text_content, &swift_syntax_tree)
                    .unwrap()
                    .unwrap();
            assert_eq!(result.0.count, 8);
            assert_eq!(result.0.function_sexp, functions[0].props.node.to_sexp());
            assert_eq!(result.0.path_from_function_root, vec![8, 1, 0, 7, 4, 0]);
            assert_eq!(result.1, 3);
        }

        #[test]
        fn extracts_expression() {
            let text_content = XcodeText::from_str(
                r#"
                import Foundation

                public func compute(input1: Int, input2: Int) -> Int {
                    if (input1 > 3 && input1 < 2) || (input2 > 4 && input2 > 6) && input1 > 0 {
                        let resolvedValue = input1 + input2 + 3;
                        let b = resolvedValue + 2
                    }
                    guard let a = input1 else {             // Guard statement prevents extraction of whole block
                        return 0;
                    }
                    if input1 > 2 {
                        return 3;
                    }
                    return 4;
                }
                
            "#,
            );

            let mut swift_syntax_tree = SwiftSyntaxTree::new();
            swift_syntax_tree.parse(&text_content).unwrap();

            let functions =
                SwiftFunction::get_top_level_functions(&swift_syntax_tree, &text_content).unwrap();
            assert_eq!(functions.len(), 1);

            let result =
                check_for_method_extraction(&functions[0], &text_content, &swift_syntax_tree)
                    .unwrap()
                    .unwrap();
            assert_eq!(result.0.count, 1);
            assert_eq!(
                result.clone().0.clone().function_sexp.clone(),
                functions[0].props.node.to_sexp()
            );
            assert_eq!(result.0.path_from_function_root, vec![10, 1, 0, 1]); // Extracts (input1 > 3 && input1 < 2) || (input2 > 4 && input2 > 6)
            assert_eq!(result.1, 4); // Remaining complexity in the function
        }
    }
}
