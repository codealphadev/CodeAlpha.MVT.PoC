use std::collections::HashMap;

use cached::proc_macro::cached;

use tree_sitter::Node;

use super::{complexity_refactoring::Edit, get_node_address, ComplexityRefactoringError};
use crate::core_engine::{
    features::complexity_refactoring::{
        refactor_function, NodeAddress, NodeSlice, SerializedNodeSlice,
    },
    rules::TemporaryFileOnDisk,
    syntax_tree::{calculate_cognitive_complexities, Complexities, SwiftFunction, SwiftSyntaxTree},
    TextPosition, XcodeText,
};
use cached::SizedCache;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tracing::debug;
use tracing::error;

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
    let possible_extractions: Vec<NodeSlice> =
        walk_node(node, text_content, syntax_tree, node_address)?;

    let function_complexity = syntax_tree
        .get_node_metadata(&node)
        .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
        .complexities
        .clone();

    const SCORE_THRESHOLD: f64 = 1.5;

    Ok(get_best_extraction(
        possible_extractions,
        syntax_tree,
        text_content,
        function_complexity.clone(),
        SCORE_THRESHOLD,
    )?
    .map(|(slice, remaining_complexity)| (slice.serialize(node), remaining_complexity)))
}

// TODO: Make async and handle error in one place, including tmp file deletion
pub fn do_method_extraction(
    slice: NodeSlice,
    set_result_callback: impl FnOnce(Vec<Edit>) -> () + Send + 'static,
    text_content: &XcodeText,
) -> Result<(), ComplexityRefactoringError> {
    let start_position = TextPosition::from_TSPoint(&slice.nodes[0].start_position());
    let range_length =
        (slice.nodes.last().unwrap().end_byte() - slice.nodes.first().unwrap().start_byte()) / 2; // UTF-16;

    // TODO: Create temporary file
    tauri::async_runtime::spawn({
        let text_content = text_content.clone();
        async move {
            let tmp_file_key = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(20)
                .map(char::from)
                .collect();
            let temp_file = match create_temp_file(&text_content, tmp_file_key) {
                Ok(res) => res,
                Err(e) => {
                    error!(?e, "Failed to create temporary file");
                    return ();
                }
            };
            let suggestion = match refactor_function(
                &temp_file.path.to_string_lossy().to_string(),
                start_position,
                range_length,
                &text_content,
            )
            .await
            {
                Err(e) => {
                    error!(?e, "Failed to query LSP for refactoring");
                    if let Err(e2) = TemporaryFileOnDisk::delete(&temp_file) {
                        error!(?e2, "In cleaning up error, could not delete temporary file");
                    }
                    return ();
                }
                Ok(Some(res)) => res,
                Ok(None) => {
                    debug!("Refactoring not possible");
                    if let Err(e2) = TemporaryFileOnDisk::delete(&temp_file) {
                        error!(?e2, "In cleaning up, could not delete temporary file");
                    }
                    return ();
                }
            };
            set_result_callback(suggestion);
        }
    });

    Ok(())
}

fn create_temp_file(
    text_content: &XcodeText,
    key: String,
) -> Result<TemporaryFileOnDisk, ComplexityRefactoringError> {
    let file_name = format!("codealpha_{}_method_extraction.swift", key);
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
    //scopes: &HashMap<NodeAddress, Scope>,
    score_threshold: f64,
) -> Result<Option<(NodeSlice<'a>, isize)>, ComplexityRefactoringError> {
    let mut best_possibility = None;
    let mut best_score = 0.0;

    let mut output_remaining_complexity: Option<isize> = None;

    // Should be higher than 1, to incentivise equalizing complexity of the two functions
    let equality_preference_factor = 1.35;

    for slice in candidates {
        //let inputs_and_outputs = slice.get_inputs_and_outputs(scopes);

        let ComplexitiesPrediction {
            removed_complexity,
            new_function_complexity,
        } = get_resulting_complexities(slice.clone(), syntax_tree, text_content)?;

        let remaining_complexity =
            (original_complexity.clone() - removed_complexity).get_total_complexity();

        let score = original_complexity.get_total_complexity() as f64
            - get_p_norm(
                remaining_complexity as f64,
                new_function_complexity.get_total_complexity() as f64,
                equality_preference_factor,
            );

        if score > best_score && score > score_threshold {
            best_possibility = Some(slice);
            best_score = score;
            output_remaining_complexity = Some(remaining_complexity);
        }
    }
    Ok(best_possibility.map(|p| {
        (
            p,
            output_remaining_complexity.expect("Remaining complexity should be set"),
        )
    }))
}

fn get_p_norm(x: f64, y: f64, exponent: f64) -> f64 {
    f64::powf(
        f64::powf(x, exponent) + f64::powf(y, exponent),
        1.0 / exponent,
    )
}

// TODO: Move scope logic into core syntax tree, and put it in metadata?
fn walk_node<'a>(
    node: Node<'a>,
    text_content: &XcodeText,
    syntax_tree: &'a SwiftSyntaxTree,
    node_address: NodeAddress,
) -> Result<Vec<NodeSlice<'a>>, ComplexityRefactoringError> {
    let mut possible_extractions: Vec<NodeSlice<'a>> = Vec::new();
    let mut cursor = node.walk();

    if node_children_are_candidates_for_extraction(&node) {
        possible_extractions.push(NodeSlice {
            nodes: node.children(&mut cursor).collect(),
            parent_address: node_address.clone(),
        });
    }

    for child in node.named_children(&mut cursor) {
        possible_extractions.append(&mut walk_node(
            child,
            text_content,
            syntax_tree,
            get_node_address(&node_address, &child),
        )?);
    }
    Ok(possible_extractions)
}

fn node_children_are_candidates_for_extraction(node: &Node) -> bool {
    node.kind() == "statements" // Restricting to blocks for now
}

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
        fn makes_correct_suggestion() {
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
    }
}
