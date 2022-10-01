use std::collections::HashMap;

use tree_sitter::Node;

use super::{
    complexity_refactoring::Edit, get_node_address, ComplexityRefactoringError, Declaration,
    DeclarationType,
};
use crate::core_engine::{
    features::complexity_refactoring::{refactor_function, NodeAddress, NodeSlice},
    syntax_tree::{calculate_cognitive_complexities, get_node_text, Complexities, SwiftSyntaxTree},
    TextPosition, XcodeText,
};
use tracing::debug;
use tracing::error;

#[derive(Clone, Debug)]
struct Scope {
    declarations: HashMap<XcodeText, Declaration>,
}

pub struct MethodExtractionOperation {
    pub edits: Vec<Edit>,
    pub prev_complexity: isize,
    pub new_complexity: isize,
}

pub fn check_for_method_extraction<'a>(
    node: Node<'a>,
    text_content: &'a XcodeText,
    syntax_tree: &'a SwiftSyntaxTree,
    file_path: &String, // TODO: Code document?
    set_result_callback: impl Fn(MethodExtractionOperation) -> () + Send + 'static,
) -> Result<(), ComplexityRefactoringError> {
    // Build up a list of possible nodes to extract, each with relevant metrics used for comparison

    let node_address = vec![node.id()];
    let mut scopes: HashMap<NodeAddress, Scope> = HashMap::new();
    scopes.insert(
        node_address.clone(),
        Scope {
            declarations: HashMap::new(),
        },
    );

    let possible_extractions: Vec<NodeSlice> =
        walk_node(node, text_content, syntax_tree, node_address, &mut scopes)?;

    let function_complexity = syntax_tree
        .get_node_metadata(&node)
        .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
        .complexities
        .clone();

    const SCORE_THRESHOLD: f64 = 1.5;

    let (best_extraction, remaining_complexity) = match get_best_extraction(
        possible_extractions,
        syntax_tree,
        text_content,
        function_complexity.clone(),
        &scopes,
        SCORE_THRESHOLD,
    )? {
        Some(r) => r,
        None => return Ok(()),
    };

    //let SliceInputsAndOutputs { inputs, outputs } = best_extraction.get_inputs_and_outputs(&scopes); // TODO: Deduplicate

    //let operation_builder = build_refactoring_operation(best_extraction, &scopes, &text_content)?;
    /*
    let input_types: Vec<DeclarationType> = inputs.into_iter().map(|input| input.1).collect();

    let resolved_return_type = match outputs.len() {
        0 => None,
        1 => Some(outputs[0].1.clone()), // TODO
        _ => {
            return Err(ComplexityRefactoringError::GenericError(anyhow!(
                "More than one output variable"
            )))
        }
    };
    */

    let slice = best_extraction;
    let start_position = TextPosition::from_TSPoint(&slice.nodes[0].start_position());
    let range_length =
        (slice.nodes.last().unwrap().end_byte() - slice.nodes.first().unwrap().start_byte()) / 2; // UTF-16;

    let prev_complexity = function_complexity.get_total_complexity();
    tauri::async_runtime::spawn({
        let file_path = file_path.clone();
        let text_content = text_content.clone();
        async move {
            let suggestion =
                match refactor_function(&file_path, start_position, range_length, &text_content)
                    .await
                {
                    Err(e) => {
                        error!(?e, "Failed to query LSP for refactoring");
                        return ();
                    }
                    Ok(Some(res)) => res,
                    Ok(None) => {
                        debug!("Refactoring not possible");
                        return ();
                    }
                };
            set_result_callback(MethodExtractionOperation {
                edits: suggestion,
                prev_complexity: prev_complexity,
                new_complexity: remaining_complexity,
            });
        }
    });

    Ok(())
}
/*
fn build_refactoring_operation(
    slice: NodeSlice,
    scopes: &HashMap<NodeAddress, Scope>,
    text_content: &XcodeText,
) -> Result<
    impl FnOnce(Vec<XcodeText>, Option<XcodeText>) -> RefactoringOperation,
    ComplexityRefactoringError,
> {
    let SliceInputsAndOutputs { inputs, outputs } = slice.get_inputs_and_outputs(scopes);

    let mut body = XcodeText::from_str("return "); // TODO: Only works for SingleExpression
    for node in &slice.nodes {
        body += get_node_text(&node, &text_content)
            .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
            + "\n";
    }

    Ok(
        move |input_types: Vec<XcodeText>, return_type: Option<XcodeText>| {
            let new_function_text = reconstruct_function(SwiftFunctionComponents {
                body,
                parameters: inputs
                    .into_iter()
                    .enumerate()
                    .map(|(i, input)| SwiftFunctionParameter {
                        external_name: input.0,
                        var_type: input_types[i].clone(),
                    })
                    .collect(),
                name: XcodeText::from_str("TODO"),
                return_type,
                context: SwiftFunctionContext::FilePrivate,
            });
            dbg!(new_function_text.clone());

            RefactoringOperation { edits: vec![] }
        },
    )
}*/
/*
#[derive(Debug, Clone)]
struct SliceInputsAndOutputs {
    inputs: Vec<(XcodeText, DeclarationType)>,
    outputs: Vec<(XcodeText, DeclarationType)>, // TODO: Make a better type
}*/
/*
// TODO: Refactor, no side effects
async fn resolve_reference_type(
    declaration_type: DeclarationType,
    file_path: &String,
) -> Result<XcodeText, ComplexityRefactoringError> {
    Ok(match declaration_type {
        DeclarationType::Unresolved(index) => {
            let resolved_type = get_type_for_index(file_path, index)
                .await
                .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?;

            XcodeText::from_str(&resolved_type)
        }
        DeclarationType::Resolved(res) => res,
    })
}*/

fn get_best_extraction<'a>(
    candidates: Vec<NodeSlice<'a>>,
    syntax_tree: &'a SwiftSyntaxTree,
    text_content: &'a XcodeText,
    original_complexity: Complexities,
    scopes: &HashMap<NodeAddress, Scope>,
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

        println!(
            "{:?}, {}, {}",
            slice.parent_address,
            slice.nodes.len(),
            score,
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
    scopes: &mut HashMap<NodeAddress, Scope>,
) -> Result<Vec<NodeSlice<'a>>, ComplexityRefactoringError> {
    // TODO: Move all the logic out of the child and into the parent

    let mut possible_extractions: Vec<NodeSlice<'a>> = Vec::new();
    for child in node.named_children(&mut node.walk()) {
        let node_address = get_node_address(&node_address, &child);

        if node_children_are_candidates_for_extraction(&child) {
            possible_extractions.push(NodeSlice {
                nodes: child.named_children(&mut child.walk()).collect(),
                parent_address: node_address.clone(),
            });
        }
        if node_has_own_scope(&child) {
            scopes.insert(
                node_address.clone(),
                Scope {
                    declarations: HashMap::new(),
                },
            );
        }

        if let Some(declaration) = try_get_declaration_node(&child) {
            let name = get_node_text(&declaration, &text_content)
                .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?;

            get_scope(&node_address, scopes).declarations.insert(
                name.clone(),
                Declaration {
                    name,
                    var_type: DeclarationType::Unresolved(declaration.start_byte() / 2), // UTF-16
                    declared_in_node: node_address.clone(),
                    referenced_in_nodes: Vec::new(),
                },
            );
        }
        if let Some(name) = get_variable_name_if_reference(&child, &text_content) {
            let mut curr_address: NodeAddress = node_address.clone();
            while curr_address.len() > 0 {
                if let Some(scope) = scopes.get_mut(&curr_address) {
                    if let Some(declaration) = scope.declarations.get_mut(&name) {
                        declaration.referenced_in_nodes.push(node_address.clone());
                        break;
                    }
                }
                curr_address.pop();
            }
        }
        possible_extractions.append(&mut walk_node(
            child,
            text_content,
            syntax_tree,
            node_address,
            scopes,
        )?);
    }
    Ok(possible_extractions)
}

fn get_scope<'a>(
    node_address: &NodeAddress,
    scopes: &'a mut HashMap<NodeAddress, Scope>,
) -> &'a mut Scope {
    let mut curr_address: NodeAddress = node_address.clone();
    while curr_address.len() > 0 {
        if scopes.get(&curr_address).is_some() {
            return scopes.get_mut(&curr_address).unwrap();
        }
        curr_address.pop();
    }
    panic!("No parent scope for node!");
}

fn node_children_are_candidates_for_extraction(node: &Node) -> bool {
    node.kind() == "statements" // Restricting to blocks for now
}

fn node_has_own_scope(node: &Node) -> bool {
    node.kind() == "statements" // TODO: Is this true??
}

fn get_variable_name_if_reference(node: &Node, text_content: &XcodeText) -> Option<XcodeText> {
    if node.kind() == "simple_identifier" {
        get_node_text(node, text_content).ok()
    } else {
        None
    }
}

// We need to track which variables were declared within each scope, because global variables should be ignored (and can't be found).
// There can be cases where we have an ERROR node etc., so just return None if no name is found
// TODO: Should we handle this differently? Maybe don't check for method extraction if an error is contained in the node. Then treat this as a real error if the assertion of simple_identifier fails.

fn try_get_declaration_node<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    let mut result: Option<(XcodeText, DeclarationType)>;

    match node.kind() {
        "property_declaration" => {
            return Some(
                node.child_by_field_name("name")?
                    .child_by_field_name("bound_identifier")?,
            );
        }
        "function_declaration" => {
            // TODO
        }
        "parameter" => {
            // Second "simple_identifier" is internal identifier, which matters; first will be overwritten
            let mut result = None;
            for child in node.children_by_field_name("name", &mut node.walk()) {
                if child.kind() == "simple_identifier" {
                    result = Some(child);
                }
            }
            return result;
        }
        "for_statement" => {
            return Some(
                node.child_by_field_name("item")?
                    .child_by_field_name("bound_identifier")?,
            );
        }
        _ => {
            // TODO: Fill in other cases.
            return None;
        }
    }

    return None;
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
/*
#[cfg(test)]
mod tests {
    mod method_extraction {

        use crate::core_engine::{
            features::complexity_refactoring::check_for_method_extraction,
            syntax_tree::SwiftSyntaxTree, XcodeText,
        };
        // TODO: Add nullish coalescing operator to complexity
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
            let tree = swift_syntax_tree.tree().unwrap();
            let root_node = tree.root_node();
            dbg!(root_node.clone().to_sexp());

            let expected_node_kinds = vec![
                "property_declaration",
                "if_statement",
                "property_declaration",
                "property_declaration",
                "for_statement",
                "assignment",
                "assignment",
                "control_transfer_statement",
            ];
            let result = None;
            check_for_method_extraction(
                root_node,
                &text_content,
                &swift_syntax_tree,
                &"file".to_string(),
                |res| result = res,
            )
            .unwrap();
            /*
            assert_eq!(
                result
                    .unwrap()
                    .iter()
                    .map(|n| n.kind().to_string())
                    .collect::<Vec<String>>(),
                expected_node_kinds
            );*/
        }
    }
}
*/
