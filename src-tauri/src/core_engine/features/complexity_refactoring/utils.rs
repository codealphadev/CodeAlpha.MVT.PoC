use tree_sitter::Node;

use super::ComplexityRefactoringError;

pub type NodeAddress = Vec<usize>;
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct NodeSlice<'a> {
    pub nodes: Vec<Node<'a>>,
    pub parent_address: NodeAddress,
}

// A serialized identifier for a NodeSlice relative to a function's s-exp which does not rely on lifetimes/IDs
#[derive(Hash, Debug, Clone, PartialEq)]
pub struct SerializedNodeSlice {
    pub path_from_function_root: Vec<usize>, // [a, b, c]: each element is the index of the descendant relative to its parent
    pub count: usize,                        // Number of nodes in the slice
    pub function_sexp: String,
}

impl<'a> NodeSlice<'a> {
    // TODO: Return error instead of panicking. But lifetime issue?
    pub fn deserialize(
        serialized_node_slice: &SerializedNodeSlice,
        function_node: Node<'a>,
    ) -> Result<NodeSlice<'a>, ComplexityRefactoringError> {
        let mut curr_node: Node<'a> = function_node;
        let mut parent_address = vec![function_node.id()];

        let mut path_to_parent = serialized_node_slice.path_from_function_root.clone();
        let first_node_index = path_to_parent
            .pop()
            .expect("path_from_function_root may not be empty");

        for index in path_to_parent {
            curr_node = curr_node.children(&mut curr_node.walk()).nth(index).ok_or(
                ComplexityRefactoringError::GenericError(anyhow!(
                    "Invalid serialized_node_slice for current function"
                )),
            )?;
            parent_address.push(curr_node.id());
        }

        let children: Vec<Node<'a>> = curr_node.children(&mut curr_node.walk()).collect();

        let nodes: Vec<Node<'a>> =
            (&children[first_node_index..first_node_index + serialized_node_slice.count]).to_vec();

        Ok(Self {
            nodes,
            parent_address,
        })
    }

    pub fn serialize(&self, function_node: Node) -> SerializedNodeSlice {
        let function_sexp = function_node.to_sexp();
        let mut curr_node = function_node;
        let mut result: Vec<usize> = vec![];
        let mut address = self.parent_address.clone();
        address.push(self.nodes[0].id());

        for node_id in address.iter().skip(1) {
            // The first element in the node address is the function's ID.

            for (i, descendant) in curr_node.children(&mut curr_node.walk()).enumerate() {
                if descendant.id() == *node_id {
                    result.push(i);
                    curr_node = descendant;
                    break;
                }
            }
        }
        return SerializedNodeSlice {
            path_from_function_root: result,
            count: self.nodes.len(),
            function_sexp,
        };
    }

    /*fn is_candidate_for_extraction(&self) -> bool {
        if nodes.iter().any(|n| n.has_error()) {
            return false;
        }
        // TODO: Check for guard statements
        return true;
    }*/
    /*
        fn get_inputs_and_outputs(
            &self,
            scopes: &HashMap<NodeAddress, Scope>,
        ) -> SliceInputsAndOutputs {
            let mut result = SliceInputsAndOutputs {
                inputs: Vec::new(),
                outputs: Vec::new(),
            };

            let mut curr_address = self.parent_address.clone();
            while curr_address.len() > 0 {
                if let Some(scope) = scopes.get(&curr_address) {
                    for declaration in scope.declarations.values() {
                        let (referenced_in_slice, referenced_in_and_after_slice) =
                            check_if_declaration_referenced_in_nodes_or_in_and_after_nodes(
                                &declaration,
                                &self.nodes,
                                &self.parent_address,
                            );
                        // TODO: Can just use one check. Doesn't matter if declaration or reference.
                        let declared_in_slice =
                            check_if_declaration_declared_in_slice(&self, &declaration);

                        let name = declaration.name.clone();

                        if declared_in_slice && referenced_in_and_after_slice {
                            result.outputs.push((name, declaration.var_type.clone()));
                        } else if referenced_in_slice && !declared_in_slice {
                            result.inputs.push((name, declaration.var_type.clone()));
                        }
                    }
                }
                curr_address.pop();
            }

            return result;
        }
    }*/
}
/*
// Checks if declaration is referenced in node range. If it is, checks if it is also referenced after it.
fn check_if_declaration_referenced_in_nodes_or_in_and_after_nodes(
    declaration: &Declaration,
    nodes: &Vec<Node>,
    surrounding_scope_address: &NodeAddress,
) -> (bool, bool) {
    let mut referenced_in_nodes = false;
    let mut referenced_after_nodes = false;
    for reference_address in &declaration.referenced_in_nodes {
        if nodes.iter().any(|n| {
            is_child_of(
                &get_node_address(surrounding_scope_address, n),
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

pub fn check_if_declaration_declared_in_slice(
    slice: &NodeSlice,
    declaration: &Declaration,
) -> bool {
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
 */
// #[derive(Debug, Clone)]
// pub enum DeclarationType {
//     Resolved(XcodeText),
//     Unresolved(usize), // Index
// }

// #[derive(Debug, Clone)]
// pub struct Declaration {
//     pub name: XcodeText,
//     pub var_type: DeclarationType, // Some types cannot be resolved in the first pass, and need to be queried from the LSP
//     pub declared_in_node: NodeAddress,
//     pub referenced_in_nodes: Vec<NodeAddress>,
// }

// fn is_child_of(parent: &NodeAddress, child: &NodeAddress) -> bool {
//     for (i, el) in parent.iter().enumerate() {
//         if child.get(i) != Some(&el) {
//             return false;
//         }
//     }
//     return true;
// }

pub fn get_node_address(parent_address: &NodeAddress, node: &Node) -> NodeAddress {
    let mut result = parent_address.clone();
    result.push(node.id());
    result
}

#[cfg(test)]
mod tests {
    mod serialization {

        use crate::core_engine::{
            features::complexity_refactoring::{NodeSlice, SerializedNodeSlice},
            syntax_tree::SwiftSyntaxTree,
            XcodeText,
        };

        use tree_sitter::Node;
        #[test]
        fn serialization_and_deserialization() {
            let serialized_node_slice;
            let original_node_kinds: Vec<String>;
            let old_function_sexp: String;
            {
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

                let function_declaration = root_node.child(0).unwrap();
                old_function_sexp = function_declaration.to_sexp();
                let function_body = function_declaration.child_by_field_name("body").unwrap();
                let statements = function_body.named_child(0).unwrap();
                let if_statement = statements.named_child(0).unwrap();
                let if_statement_2 = if_statement.named_child(4).unwrap();
                let statements_2 = if_statement_2.named_child(2).unwrap();

                let statements_node = root_node
                    .child(0)
                    .unwrap() // function_declaration
                    .child_by_field_name("body")
                    .unwrap() // function_body
                    .named_child(0)
                    .unwrap() // statements
                    .named_child(0)
                    .unwrap() // if_statement
                    .named_child(4)
                    .unwrap() // if_statement
                    .named_child(2)
                    .unwrap(); // statements
                assert_eq!(statements_node.kind(), "statements");
                // Child nodes:
                // property_declaration, if_statement, property_declaration, property_declaration, for_statement, assignment, assignment, control_transfer_statement

                let parent_address = [
                    function_declaration.id(),
                    function_body.id(),
                    statements.id(),
                    if_statement.id(),
                    if_statement_2.id(),
                    statements_2.id(),
                ];
                let node_slice = NodeSlice {
                    nodes: statements_node
                        .children(&mut statements_node.walk())
                        .collect::<Vec<Node>>()[1..7] // all except first and last
                        .to_vec(),
                    parent_address: parent_address.to_vec(),
                };
                serialized_node_slice = node_slice.serialize(function_declaration);
                assert_eq!(
                    serialized_node_slice,
                    SerializedNodeSlice {
                        path_from_function_root: vec![8, 1, 0, 7, 4, 1],
                        count: 6,
                        function_sexp: old_function_sexp
                    }
                );
                original_node_kinds = node_slice
                    .nodes
                    .iter()
                    .map(|n| n.kind().to_string())
                    .collect::<Vec<String>>();
            }
            //Modify the text content, replacing 'c' with 'coo' etc.
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
                        var aoo: Int = 0;
                        var boo = 1;
                        for i in 1..<(Int(input) ?? 0) {                // + 2 (1 for nesting)
                            let coo = aoo + boo;
                            aoo = boo;
                            boo = coo;
                        }
                        result = coo;
                        result = boo;
                        return String(boo);
                    } else {                                            // + 1
                        return "undefined";
                    }
                }
            "#,
            );
            // Re-parse tree from scratch
            let mut swift_syntax_tree = SwiftSyntaxTree::new();
            swift_syntax_tree.parse(&text_content).unwrap();
            let tree = swift_syntax_tree.tree().unwrap();
            let root_node = tree.root_node();

            let function_declaration = root_node.child(0).unwrap();

            let recovered_slice =
                NodeSlice::deserialize(&serialized_node_slice, function_declaration).unwrap();

            let reserialized_slice = recovered_slice.serialize(function_declaration);
            assert_eq!(reserialized_slice, serialized_node_slice);
            assert_eq!(
                recovered_slice
                    .nodes
                    .iter()
                    .map(|n| n.kind().to_string())
                    .collect::<Vec<String>>(),
                original_node_kinds
            );
        }
    }
    // mod is_child_of {
    //     use crate::core_engine::features::complexity_refactoring::utils::is_child_of;

    //     #[test]
    //     fn equal_case() {
    //         let parent = vec![22, 54, 25];
    //         let child = vec![22, 54, 25];
    //         assert_eq!(is_child_of(&parent, &child), true);
    //     }

    //     #[test]
    //     fn unequal_case() {
    //         let parent = vec![22, 54, 25];
    //         let child = vec![22, 51, 25];
    //         assert_eq!(is_child_of(&parent, &child), false);
    //     }

    //     #[test]
    //     fn contains_case() {
    //         let parent = vec![22, 54, 25];
    //         let child = vec![22, 54, 25, 39, 12, 63];
    //         assert_eq!(is_child_of(&parent, &child), true);
    //     }

    //     #[test]
    //     fn reverse_case() {
    //         let parent = vec![22, 51, 25, 39, 12, 63];
    //         let child = vec![22, 54, 25];
    //         assert_eq!(is_child_of(&parent, &child), false);
    //     }

    //     #[test]
    //     fn empty_parent_case() {
    //         let parent = vec![];
    //         let child = vec![22, 54, 25];
    //         assert_eq!(is_child_of(&parent, &child), true);
    //     }

    //     #[test]
    //     fn empty_child_case() {
    //         let parent = vec![124];
    //         let child = vec![];
    //         assert_eq!(is_child_of(&parent, &child), false);
    //     }

    //     #[test]
    //     fn empty_case() {
    //         let parent = vec![];
    //         let child = vec![];
    //         assert_eq!(is_child_of(&parent, &child), true);
    //     }
    // }
}
