use std::{collections::HashMap, fmt};

use tree_sitter::Node;

use crate::core_engine::{
    syntax_tree::{calculate_cognitive_complexities, get_node_text, Complexities, SwiftSyntaxTree},
    XcodeText,
};

use super::ComplexityRefactoringError;

#[derive(Clone, Debug)]
struct Scope {
    declarations: HashMap<XcodeText, Declaration>,
}

#[derive(Debug, Clone)]
struct NodeSlice<'a> {
    nodes: Vec<Node<'a>>,
    parent_address: NodeAddress,
}

pub fn check_for_method_extraction<'a>(
    node: Node<'a>,
    text_content: &'a XcodeText,
    syntax_tree: &'a SwiftSyntaxTree,
) -> Result<Option<Vec<Node<'a>>>, ComplexityRefactoringError> {
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

    let (best_extraction, score) = get_best_extraction(
        possible_extractions,
        syntax_tree,
        text_content,
        function_complexity,
        &scopes,
    )?;

    const SCORE_THRESHOLD: f64 = 1.0;

    if score > SCORE_THRESHOLD {
        return Ok(best_extraction.map(|e| e.nodes));
    } else {
        return Ok(None);
    }
}

#[derive(Debug, Clone)]
struct ReferencesInSlice {
    input_names: Vec<XcodeText>,
    output_names: Vec<XcodeText>,
}

impl<'a> NodeSlice<'a> {
    /*fn is_candidate_for_extraction(&self) -> bool {
        if nodes.iter().any(|n| n.has_error()) {
            return false;
        }
        // TODO: Check for guard statements
        return true;
    }*/

    fn classify_references_in_slice(
        &self,
        scopes: &HashMap<NodeAddress, Scope>,
    ) -> ReferencesInSlice {
        let mut result = ReferencesInSlice {
            output_names: Vec::new(),
            input_names: Vec::new(),
        };

        let mut curr_address = self.parent_address.clone();
        while curr_address.len() > 0 {
            if let Some(scope) = scopes.get(&curr_address) {
                for (name, declaration) in &scope.declarations {
                    let (referenced_in_slice, referenced_in_and_after_slice) =
                        check_if_declaration_referenced_in_nodes_or_in_and_after_nodes(
                            &declaration,
                            &self.nodes,
                            &self.parent_address,
                        );
                    // TODO: Can just use one check. Doesn't matter if declaration or reference.
                    let declared_in_slice =
                        check_if_declaration_declared_in_slice(&self, &declaration);

                    if declared_in_slice && referenced_in_and_after_slice {
                        result.output_names.push(name.clone());
                    } else if referenced_in_slice && !declared_in_slice {
                        result.input_names.push(name.clone());
                    }
                }
            }
            curr_address.pop();
        }
        return result;
    }
}

fn check_if_declaration_declared_in_slice(slice: &NodeSlice, declaration: &Declaration) -> bool {
    if slice.nodes.iter().any(|n| {
        is_child_of(
            &get_node_address(&slice.parent_address, n),
            &declaration.declared_in_node,
        )
    }) {
        return true;
    }
    false
}

// Checks if declaration is referenced in node range. If it is, checks if it is also referenced after it.
fn check_if_declaration_referenced_in_nodes_or_in_and_after_nodes(
    declaration: &Declaration,
    nodes: &Vec<Node>,
    surrounding_scope_address: &NodeAddress,
) -> (bool, bool) {
    let mut referenced_in_nodes = false;
    let mut referenced_after_nodes = false;
    for reference in &declaration.referenced_in_nodes {
        if nodes
            .iter()
            .any(|n| is_child_of(&get_node_address(surrounding_scope_address, n), &reference))
        {
            referenced_in_nodes = true;
        } else if referenced_in_nodes {
            referenced_after_nodes = true;
        }
    }
    (referenced_in_nodes, referenced_after_nodes)
}

fn get_node_address(parent_address: &NodeAddress, node: &Node) -> NodeAddress {
    let mut result = parent_address.clone();
    result.push(node.id());
    result
}

fn get_best_extraction<'a>(
    candidates: Vec<NodeSlice<'a>>,
    syntax_tree: &'a SwiftSyntaxTree,
    text_content: &'a XcodeText,
    original_complexity: Complexities,
    scopes: &HashMap<NodeAddress, Scope>,
) -> Result<(Option<NodeSlice<'a>>, f64), ComplexityRefactoringError> {
    let mut best_possibility = None;
    let mut best_score = 0.0;

    // Should be higher than 1, to incentivise equalizing complexity of the two functions
    let equality_preference_factor = 1.35;

    for slice in candidates {
        let inputs_and_outputs = slice.classify_references_in_slice(scopes);

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
            "{:?}, {}, {}, {:?}",
            slice.parent_address,
            slice.nodes.len(),
            score,
            inputs_and_outputs
        );
        if score > best_score {
            best_possibility = Some(slice);
            best_score = score;
        }
    }
    Ok((best_possibility, best_score))
}

fn get_p_norm(x: f64, y: f64, exponent: f64) -> f64 {
    f64::powf(
        f64::powf(x, exponent) + f64::powf(y, exponent),
        1.0 / exponent,
    )
}

type NodeAddress = Vec<usize>;

fn is_child_of(parent: &NodeAddress, child: &NodeAddress) -> bool {
    for (i, el) in parent.iter().enumerate() {
        if child.get(i) != Some(&el) {
            return false;
        }
    }
    return true;
}
#[derive(Debug, Clone)]
struct Declaration {
    declared_in_node: NodeAddress,
    referenced_in_nodes: Vec<NodeAddress>,
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
        if let Some(name) = get_variable_name_if_declaration(&child, &text_content) {
            get_scope(&node_address, scopes).declarations.insert(
                name,
                Declaration {
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
fn get_variable_name_if_declaration(node: &Node, text_content: &XcodeText) -> Option<XcodeText> {
    match node.kind() {
        "property_declaration" => {
            let name_node = node
                .child_by_field_name("name")?
                .child_by_field_name("bound_identifier")?;
            return get_node_text(&name_node, &text_content).ok();
        }
        "function_declaration" => {
            // TODO
        }
        "parameter" => {
            for child in node.children_by_field_name("name", &mut node.walk()) {
                if child.kind() == "simple_identifier" {
                    return get_node_text(&child, &text_content).ok();
                }
            }
        }
        "for_statement" => {
            let name_node = node
                .child_by_field_name("item")?
                .child_by_field_name("bound_identifier")?;

            // There can be cases where we have an ERROR node etc., so just return None if no name is found
            // TODO: Should we handle this differently? Maybe don't check for method extraction if an error is contained in the node. Then treat this as a real error if the assertion of simple_identifier fails.
            if name_node.kind() == "simple_identifier" {
                return get_node_text(&name_node, &text_content).ok();
            }
        }
        _ => (),
    }
    // TODO: Fill in other cases.
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

#[cfg(test)]
mod tests {
    mod is_child_of {
        use crate::core_engine::features::complexity_refactoring::method_extraction::is_child_of;

        #[test]
        fn equal_case() {
            let parent = vec![22, 54, 25];
            let child = vec![22, 54, 25];
            assert_eq!(is_child_of(&parent, &child), true);
        }

        #[test]
        fn unequal_case() {
            let parent = vec![22, 54, 25];
            let child = vec![22, 51, 25];
            assert_eq!(is_child_of(&parent, &child), false);
        }

        #[test]
        fn contains_case() {
            let parent = vec![22, 54, 25];
            let child = vec![22, 54, 25, 39, 12, 63];
            assert_eq!(is_child_of(&parent, &child), true);
        }

        #[test]
        fn reverse_case() {
            let parent = vec![22, 51, 25, 39, 12, 63];
            let child = vec![22, 54, 25];
            assert_eq!(is_child_of(&parent, &child), false);
        }

        #[test]
        fn empty_parent_case() {
            let parent = vec![];
            let child = vec![22, 54, 25];
            assert_eq!(is_child_of(&parent, &child), true);
        }

        #[test]
        fn empty_child_case() {
            let parent = vec![124];
            let child = vec![];
            assert_eq!(is_child_of(&parent, &child), false);
        }

        #[test]
        fn empty_case() {
            let parent = vec![];
            let child = vec![];
            assert_eq!(is_child_of(&parent, &child), true);
        }
    }
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
                        var a = 0;
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

            let result =
                check_for_method_extraction(root_node, &text_content, &swift_syntax_tree).unwrap();

            assert_eq!(
                result
                    .unwrap()
                    .iter()
                    .map(|n| n.kind().to_string())
                    .collect::<Vec<String>>(),
                expected_node_kinds
            );
        }
    }
}
