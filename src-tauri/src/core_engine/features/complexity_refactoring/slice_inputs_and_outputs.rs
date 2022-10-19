use std::collections::HashMap;

use tree_sitter::Node;

use crate::core_engine::{syntax_tree::get_node_text, XcodeText};

use super::{
    get_node_address, is_child_of, ComplexityRefactoringError, NodeAddress, NodeSlice, NodeSubSlice,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    pub declarations: HashMap<XcodeText, Declaration>,
}

#[derive(Clone, Debug)]
pub struct ContinueOrBreak {
    pub node_address: NodeAddress,
    pub target_node_address: NodeAddress,
}

pub struct ParsingMetadata {
    pub scopes: HashMap<NodeAddress, Scope>,
    pub continues_and_breaks: Vec<ContinueOrBreak>,
}

impl ParsingMetadata {
    pub fn new(function_node_address: NodeAddress) -> Self {
        Self {
            scopes: HashMap::from([(
                function_node_address,
                Scope {
                    declarations: HashMap::new(),
                },
            )]),
            continues_and_breaks: Vec::new(),
        }
    }
}
pub fn update_parsing_metadata_for_node(
    parsing_metadata: &mut ParsingMetadata,
    node: &Node,
    node_address: &NodeAddress,
    text_content: &XcodeText,
) -> Result<(), ComplexityRefactoringError> {
    if node_has_own_scope(&node) {
        parsing_metadata.scopes.insert(
            node_address.clone(),
            Scope {
                declarations: HashMap::new(),
            },
        );
    }

    if let Some(declaration) = try_get_declaration_node(&node) {
        let name = get_node_text(&declaration, &text_content)
            .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?;

        get_scope(&node_address, &mut parsing_metadata.scopes)
            .declarations
            .insert(
                name.clone(),
                Declaration {
                    declared_in_node: node_address.clone(),
                    referenced_in_nodes: Vec::new(),
                },
            );
    }
    if let Some(name) = get_variable_name_if_reference(&node, &text_content) {
        let mut curr_address: NodeAddress = node_address.clone();
        while curr_address.len() > 0 {
            if let Some(scope) = parsing_metadata.scopes.get_mut(&curr_address) {
                if let Some(declaration) = scope.declarations.get_mut(&name) {
                    declaration.referenced_in_nodes.push(node_address.clone());
                    break;
                }
            }
            curr_address.pop();
        }
    }
    if node.kind() == "control_transfer_statement" {
        let child_kind = node.child(0).map(|n| n.kind());
        if child_kind == Some("continue") || child_kind == Some("break") {
            let target_label = node
                .child_by_field_name("result")
                .and_then(|n| get_node_text(&n, text_content).ok());

            let target_address = get_target_for_continue_or_break_statement(
                node.clone(),
                &node_address,
                target_label,
                text_content,
            );
            if let Some(target_node_address) = target_address {
                parsing_metadata.continues_and_breaks.push(ContinueOrBreak {
                    node_address: node_address.clone(),
                    target_node_address,
                })
            }
        }
    }
    Ok(())
}

fn get_target_for_continue_or_break_statement(
    node: Node,
    node_address: &NodeAddress,
    target_label: Option<XcodeText>,
    text_content: &XcodeText,
) -> Option<NodeAddress> {
    let mut curr_address = node_address.clone();
    let mut curr_node = node;
    while curr_address.len() > 0 {
        curr_address.pop();
        curr_node = curr_node.parent()?;

        let curr_node_kind = curr_node.kind();
        if curr_node_kind == "for_statement"
            || curr_node_kind == "while_statement"
            || curr_node_kind == "repeat_while_statement"
        {
            if let Some(ref label) = target_label {
                if let Some(prev_sibling) = curr_node.prev_named_sibling() {
                    let label_node_str = get_node_text(&prev_sibling, text_content)
                        .ok()?
                        .as_string()
                        .replace(":", "")
                        .replace(" ", "");
                    if prev_sibling.kind() == "statement_label"
                        && label_node_str == label.as_string()
                    {
                        return Some(curr_address);
                    }
                }
            } else {
                return Some(curr_address);
            }
        }
    }
    return None;
}

#[derive(PartialEq, Clone, Debug)]
pub struct SliceInputsAndOutputs {
    pub input_names: Vec<XcodeText>,
    pub output_names: Vec<XcodeText>,
}

pub fn get_slice_inputs_and_outputs(
    slice: &NodeSlice,
    parsing_metadata: &ParsingMetadata,
) -> SliceInputsAndOutputs {
    get_slice_inputs_and_outputs_internal(
        &slice.nodes.iter().map(|n| n.id()).collect(),
        &slice.parent_address,
        &parsing_metadata.scopes,
    )
}

pub fn get_sub_slice_inputs_and_outputs(
    slice: &NodeSubSlice,
    parsing_metadata: &ParsingMetadata,
) -> SliceInputsAndOutputs {
    get_slice_inputs_and_outputs_internal(
        &slice.nodes.iter().map(|n| n.id()).collect(),
        &slice.parent_address,
        &parsing_metadata.scopes,
    )
}
fn get_slice_inputs_and_outputs_internal(
    node_ids: &Vec<usize>,
    parent_address: &NodeAddress,
    scopes: &HashMap<NodeAddress, Scope>,
) -> SliceInputsAndOutputs {
    let mut result = SliceInputsAndOutputs {
        input_names: Vec::new(),
        output_names: Vec::new(),
    };
    let mut curr_address = parent_address.clone();
    while curr_address.len() > 0 {
        if let Some(scope) = scopes.get(&curr_address) {
            for (name, declaration) in &scope.declarations {
                let (referenced_in_slice, referenced_in_and_after_slice) =
                    check_if_declaration_referenced_in_nodes_or_in_and_after_nodes(
                        &declaration,
                        &node_ids,
                        &parent_address,
                    );
                // TODO: Can just use one check. Doesn't matter if declaration or reference.
                let declared_in_slice = check_if_declaration_declared_in_slice(
                    &node_ids,
                    &parent_address,
                    &declaration,
                );
                let name = name.clone();
                if declared_in_slice && referenced_in_and_after_slice {
                    result.output_names.push(name);
                } else if referenced_in_slice && !declared_in_slice {
                    result.input_names.push(name);
                }
            }
        }
        curr_address.pop();
    }
    return result;
}

fn node_has_own_scope(node: &Node) -> bool {
    node.kind() == "statements" // TODO: Fix this. Is it true??
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

fn get_variable_name_if_reference(node: &Node, text_content: &XcodeText) -> Option<XcodeText> {
    if node.kind() == "simple_identifier" {
        get_node_text(node, text_content).ok()
    } else {
        None
    }
}

fn try_get_declaration_node<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    match node.kind() {
        "property_declaration" => {
            return Some(
                node.child_by_field_name("name")?
                    .child_by_field_name("bound_identifier")?,
            );
        }
        "function_declaration" => {
            // Note: This will also pick up the main function name, treating it as an 'input', even though it is globally available. TODO: Fix this at some point - although it's not too bad to penalise recursion triangles...
            for child in node.children_by_field_name("name", &mut node.walk()) {
                if child.kind() == "simple_identifier" {
                    return Some(child);
                }
            }
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
            // Not sure if there are any other cases?
            return None;
        }
    }

    return None;
}

// Checks if declaration is referenced in node range. If it is, checks if it is also referenced after it.
fn check_if_declaration_referenced_in_nodes_or_in_and_after_nodes(
    declaration: &Declaration,
    node_ids: &Vec<usize>,
    surrounding_scope_address: &NodeAddress,
) -> (bool, bool) {
    let mut referenced_in_nodes = false;
    let mut referenced_after_nodes = false;
    for reference_address in &declaration.referenced_in_nodes {
        if node_ids.iter().any(|node_id| {
            is_child_of(
                &get_node_address(surrounding_scope_address, *node_id),
                &reference_address,
            )
        }) {
            referenced_in_nodes = true;
        } else if referenced_in_nodes {
            referenced_after_nodes = true;
        }
    }
    // Check if there is a return statement within our slice that returns the node.
    (referenced_in_nodes, referenced_after_nodes)
}
fn check_if_declaration_declared_in_slice(
    node_ids: &Vec<usize>,
    parent_address: &NodeAddress,
    declaration: &Declaration,
) -> bool {
    if node_ids.iter().any(|node_id| {
        is_child_of(
            &get_node_address(&parent_address, *node_id),
            &declaration.declared_in_node,
        )
    }) {
        return true;
    }
    false
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub declared_in_node: NodeAddress,
    pub referenced_in_nodes: Vec<NodeAddress>,
}

#[cfg(test)]
mod tests {
    mod update_parsing_metadata_for_node {

        use tree_sitter::Node;

        use crate::core_engine::{
            features::complexity_refactoring::{
                get_node_address, update_parsing_metadata_for_node, NodeAddress, ParsingMetadata,
            },
            syntax_tree::{SwiftFunction, SwiftSyntaxTree},
            XcodeText,
        };

        fn walk_node_test<'a>(
            node: Node<'a>,
            text_content: &XcodeText,
            syntax_tree: &'a SwiftSyntaxTree,
            node_address: NodeAddress,
            parsing_metadata: &mut ParsingMetadata,
        ) -> () {
            update_parsing_metadata_for_node(parsing_metadata, &node, &node_address, &text_content)
                .unwrap();

            for child in node.named_children(&mut node.walk()) {
                walk_node_test(
                    child,
                    text_content,
                    syntax_tree,
                    get_node_address(&node_address, child.id()),
                    parsing_metadata,
                );
            }
        }

        #[test]
        fn handles_continues_and_breaks() {
            let text_content = XcodeText::from_str(
                r#"
                func function(arg1: Int) {
                    for i in 1...3 {
                        if i == 2 {
                            break;
                        }
                    }
                }
            "#,
            );
            let mut syntax_tree = SwiftSyntaxTree::new(SwiftSyntaxTree::parser_mutex());
            syntax_tree.parse(&text_content).unwrap();

            let functions =
                SwiftFunction::get_top_level_functions(&syntax_tree, &text_content).unwrap();
            assert_eq!(functions.len(), 1);

            let function_decl_node = functions[0].props.node;
            let function_node_address = vec![function_decl_node.clone().id()];

            let mut parsing_metadata = ParsingMetadata::new(function_node_address.clone());
            walk_node_test(
                function_decl_node,
                &text_content,
                &syntax_tree,
                function_node_address.clone(),
                &mut parsing_metadata,
            );
            assert_eq!(parsing_metadata.continues_and_breaks.clone().len(), 1);

            let statement = parsing_metadata.continues_and_breaks[0].clone();
            assert_eq!(statement.node_address.len(), 8); // function_decl, function_body, statements, for_statement, statements, if_statement, statements, control_transfer_statement
            assert_eq!(statement.target_node_address.len(), 4); // function_decl, function_body, statements, for_statement
        }

        #[test]
        fn handles_while_loops_and_continues() {
            let text_content = XcodeText::from_str(
                r#"
                func function(arg1: Int) {
                    for i in 1...2 {
                        while true {
                            continue;
                        }
                    }
                }
            "#,
            );
            let mut syntax_tree = SwiftSyntaxTree::new(SwiftSyntaxTree::parser_mutex());
            syntax_tree.parse(&text_content).unwrap();

            let functions =
                SwiftFunction::get_top_level_functions(&syntax_tree, &text_content).unwrap();
            assert_eq!(functions.len(), 1);

            let function_decl_node = functions[0].props.node;
            let function_node_address = vec![function_decl_node.clone().id()];

            let mut parsing_metadata = ParsingMetadata::new(function_node_address.clone());
            walk_node_test(
                function_decl_node,
                &text_content,
                &syntax_tree,
                function_node_address.clone(),
                &mut parsing_metadata,
            );
            assert_eq!(parsing_metadata.continues_and_breaks.clone().len(), 1);

            let statement = parsing_metadata.continues_and_breaks[0].clone();
            assert_eq!(statement.node_address.len(), 8); // function_decl, function_body, statements, for_statement, statements, while_statement, statements, control_transfer_statement
            assert_eq!(statement.target_node_address.len(), 6); // function_decl, function_body, statements, for_statement, statements, while_statement
        }

        #[test]
        fn handles_labeled_loops() {
            let text_content = XcodeText::from_str(
                r#"
                func function(arg1: Int) {
                    labelly: repeat {
                        while true {
                            break labelly;
                        }
                    } while true;
                    return;
                }
            "#,
            );
            let mut syntax_tree = SwiftSyntaxTree::new(SwiftSyntaxTree::parser_mutex());
            syntax_tree.parse(&text_content).unwrap();

            let functions =
                SwiftFunction::get_top_level_functions(&syntax_tree, &text_content).unwrap();
            assert_eq!(functions.len(), 1);

            let function_decl_node = functions[0].props.node;
            let function_node_address = vec![function_decl_node.clone().id()];

            let mut parsing_metadata = ParsingMetadata::new(function_node_address.clone());
            walk_node_test(
                function_decl_node,
                &text_content,
                &syntax_tree,
                function_node_address.clone(),
                &mut parsing_metadata,
            );
            assert_eq!(parsing_metadata.continues_and_breaks.clone().len(), 1);

            let statement = parsing_metadata.continues_and_breaks[0].clone();
            assert_eq!(statement.node_address.len(), 8); // function_decl, function_body, statements, repeat_while_statement, statements, while_statement, statements, control_transfer_statement
            assert_eq!(statement.target_node_address.len(), 4); // function_decl, function_body, statements, repeat_while_statement
        }

        #[test]
        fn builds_up_correct_scope_tree() {
            let text_content = XcodeText::from_str(
                r#"
                func function(arg1: Int) {
                    let newNum = arg1 + externalArg;
                    return newNum;
                }
            "#,
            );
            let mut syntax_tree = SwiftSyntaxTree::new(SwiftSyntaxTree::parser_mutex());
            syntax_tree.parse(&text_content).unwrap();

            let functions =
                SwiftFunction::get_top_level_functions(&syntax_tree, &text_content).unwrap();
            assert_eq!(functions.len(), 1);

            let function_decl_node = functions[0].props.node;
            let function_node_address = vec![function_decl_node.clone().id()];

            let mut parsing_metadata = ParsingMetadata::new(function_node_address.clone());
            walk_node_test(
                function_decl_node,
                &text_content,
                &syntax_tree,
                function_node_address.clone(),
                &mut parsing_metadata,
            );

            assert_eq!(parsing_metadata.continues_and_breaks.len(), 0);
            assert_eq!(parsing_metadata.scopes.len(), 2); // Starting scope, statements scope
            let function_scope_decls = parsing_metadata
                .scopes
                .get(&vec![function_decl_node.id()])
                .unwrap()
                .declarations
                .clone();
            assert_eq!(function_scope_decls.len(), 2); // TODO: Should really be 1, but it's not too bad to also include the main function name here as a false input (penalising triangular recursion implicitly)
            let arg1_decl = function_scope_decls
                .get(&XcodeText::from_str("arg1"))
                .unwrap();
            assert_eq!(arg1_decl.declared_in_node.len(), 2); // function_decl, parameter
            assert_eq!(arg1_decl.referenced_in_nodes.len(), 2); // Declared once, used in one other place
            assert_eq!(arg1_decl.referenced_in_nodes[0].len(), 3); // function_decl, parameter, simple_expression
            assert_eq!(arg1_decl.referenced_in_nodes[1].len(), 6); // function_decl, function_body, statements, property_declaration, additive_expression, simple_identifier

            parsing_metadata
                .scopes
                .remove(&vec![function_decl_node.id()]);
            assert_eq!(parsing_metadata.scopes.keys().len(), 1);
            assert_eq!(parsing_metadata.scopes.keys().next().unwrap().len(), 3); // function_decl, function_body, statements
            assert_eq!(
                parsing_metadata
                    .scopes
                    .values()
                    .next()
                    .unwrap()
                    .declarations
                    .len(),
                1
            );

            let statements_scope_decls = parsing_metadata
                .scopes
                .values()
                .next()
                .unwrap()
                .declarations
                .clone();
            let newNum_decl = statements_scope_decls
                .get(&XcodeText::from_str("newNum"))
                .unwrap();

            assert_eq!(newNum_decl.declared_in_node.len(), 4); // function_decl, function_body_statements, property_declaration
            assert_eq!(newNum_decl.referenced_in_nodes.len(), 2); // Declared once, referenced once
            assert_eq!(newNum_decl.referenced_in_nodes[0].len(), 6); // function_decl, function_body, statements, property_declaration, additive_expression, simple_identifier
        }
    }

    mod get_inputs_and_outputs {
        use std::collections::HashMap;

        use crate::core_engine::{
            features::complexity_refactoring::{
                slice_inputs_and_outputs::get_slice_inputs_and_outputs_internal, Declaration,
                NodeAddress, Scope, SliceInputsAndOutputs,
            },
            XcodeText,
        };

        #[test]
        fn no_inputs_and_outputs() {
            let var_x = XcodeText::from_str("x");
            let scopes: HashMap<NodeAddress, Scope> = HashMap::from([(
                vec![1],
                Scope {
                    declarations: HashMap::from([(
                        var_x,
                        Declaration {
                            declared_in_node: vec![1, 15],
                            referenced_in_nodes: vec![vec![1, 15], vec![1, 16]],
                        },
                    )]),
                },
            )]);
            let slice_node_ids = vec![15, 16, 17];
            let parent_address = vec![1];
            assert_eq!(
                get_slice_inputs_and_outputs_internal(&slice_node_ids, &parent_address, &scopes),
                SliceInputsAndOutputs {
                    input_names: vec![],
                    output_names: vec![]
                }
            )
        }

        #[test]
        fn used_in_block_after_declaration_in_slice() {
            let var_x = XcodeText::from_str("x");
            let scopes: HashMap<NodeAddress, Scope> = HashMap::from([(
                vec![1],
                Scope {
                    declarations: HashMap::from([(
                        var_x.clone(),
                        Declaration {
                            declared_in_node: vec![1, 15],
                            referenced_in_nodes: vec![vec![1, 15], vec![1, 16], vec![1, 17]],
                        },
                    )]),
                },
            )]);
            let slice_node_ids = vec![15, 16];
            let parent_address = vec![1];
            assert_eq!(
                get_slice_inputs_and_outputs_internal(&slice_node_ids, &parent_address, &scopes),
                SliceInputsAndOutputs {
                    input_names: vec![],
                    output_names: vec![var_x]
                }
            )
        }

        #[test]
        fn used_only_before_and_after_slice() {
            let var_x = XcodeText::from_str("x");
            let scopes: HashMap<NodeAddress, Scope> = HashMap::from([(
                vec![1],
                Scope {
                    declarations: HashMap::from([(
                        var_x.clone(),
                        Declaration {
                            declared_in_node: vec![1, 15],
                            referenced_in_nodes: vec![vec![1, 15], vec![1, 17]],
                        },
                    )]),
                },
            )]);
            let slice_node_ids = vec![16];
            let parent_address = vec![1];
            assert_eq!(
                get_slice_inputs_and_outputs_internal(&slice_node_ids, &parent_address, &scopes),
                SliceInputsAndOutputs {
                    input_names: vec![],
                    output_names: vec![]
                }
            )
        }

        #[test]
        fn used_before_and_in_slice() {
            let var_x = XcodeText::from_str("x");
            let scopes: HashMap<NodeAddress, Scope> = HashMap::from([(
                vec![1],
                Scope {
                    declarations: HashMap::from([(
                        var_x.clone(),
                        Declaration {
                            declared_in_node: vec![1, 14],
                            referenced_in_nodes: vec![vec![1, 15], vec![1, 16]],
                        },
                    )]),
                },
            )]);
            let slice_node_ids = vec![15, 16, 17];
            let parent_address = vec![1];
            assert_eq!(
                get_slice_inputs_and_outputs_internal(&slice_node_ids, &parent_address, &scopes),
                SliceInputsAndOutputs {
                    input_names: vec![var_x],
                    output_names: vec![]
                }
            )
        }

        #[test]
        fn declaration_in_outer_scope_used_in_slice() {
            // A declaration in the outer scope of a function also needs to be extracted if used in the slice
            let var_x = XcodeText::from_str("x");
            let scopes: HashMap<NodeAddress, Scope> = HashMap::from([(
                vec![1],
                Scope {
                    declarations: HashMap::from([(
                        var_x.clone(),
                        Declaration {
                            declared_in_node: vec![1, 14],
                            referenced_in_nodes: vec![vec![1, 15, 151], vec![1, 15, 152]],
                        },
                    )]),
                },
            )]);
            let slice_node_ids = vec![151, 152];
            let parent_address = vec![1, 15];

            assert_eq!(
                get_slice_inputs_and_outputs_internal(&slice_node_ids, &parent_address, &scopes),
                SliceInputsAndOutputs {
                    input_names: vec![var_x],
                    output_names: vec![]
                }
            )
        }
    }
}
