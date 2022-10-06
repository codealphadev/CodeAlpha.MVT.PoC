use tree_sitter::Node;

use super::{ComplexityRefactoringError, NodeAddress};

use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct NodeSlice<'a> {
    pub nodes: Vec<Node<'a>>,
    pub parent_address: NodeAddress,
}

pub struct NodeSubSlice<'a> {
    pub nodes: &'a [Node<'a>],
    pub parent_address: &'a NodeAddress,
}

// A serialized identifier for a NodeSlice relative to a function's s-exp which does not rely on lifetimes/IDs
#[derive(Hash, Debug, Clone, PartialEq)]
pub struct SerializedNodeSlice {
    pub path_from_function_root: Vec<usize>, // [a, b, c]: each element is the index of the descendant relative to its parent
    pub count: usize,                        // Number of nodes in the slice
    pub function_sexp: String,
}

impl<'a> NodeSlice<'a> {
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
}
