use std::collections::HashMap;

use tree_sitter::Node;

use super::{
    complexity_refactoring::Edit, get_node_address, ComplexityRefactoringError, Declaration,
    DeclarationType,
};
use crate::core_engine::{
    features::complexity_refactoring::{refactor_function, NodeAddress, NodeSlice},
    syntax_tree::{
        calculate_cognitive_complexities, get_node_text, Complexities, SwiftFunction,
        SwiftSyntaxTree,
    },
    TextPosition, XcodeText,
};
use tracing::debug;
use tracing::error;

#[derive(Clone, Debug)]
struct Scope {
    declarations: HashMap<XcodeText, Declaration>,
}

pub fn check_for_method_extraction<'a>(
    function: &SwiftFunction<'a>,
    text_content: &'a XcodeText,
    syntax_tree: &'a SwiftSyntaxTree,
) -> Result<Option<(NodeSlice<'a>, isize)>, ComplexityRefactoringError> {
    let node = function.props.node;
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

    get_best_extraction(
        possible_extractions,
        syntax_tree,
        text_content,
        function_complexity.clone(),
        &scopes,
        SCORE_THRESHOLD,
    )
}

pub fn do_method_extraction(
    slice: NodeSlice,
    set_result_callback: impl FnOnce(Vec<Edit>) -> () + Send + 'static,
    text_content: &XcodeText,
    file_path: &String, // TODO: Code document? // TODO: Create temporary file
) -> Result<(), ComplexityRefactoringError> {
    let start_position = TextPosition::from_TSPoint(&slice.nodes[0].start_position());
    let range_length =
        (slice.nodes.last().unwrap().end_byte() - slice.nodes.first().unwrap().start_byte()) / 2; // UTF-16;

    // TODO: Create temporary file
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
            set_result_callback(suggestion);
        }
    });

    Ok(())
}

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
                nodes: child.children(&mut child.walk()).collect(),
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
