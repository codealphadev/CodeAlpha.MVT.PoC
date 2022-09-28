use tree_sitter::Node;

use crate::core_engine::XcodeText;

pub type NodeAddress = Vec<usize>;

#[derive(Debug, Clone)]
pub struct NodeSlice<'a> {
    pub nodes: Vec<Node<'a>>,
    pub parent_address: NodeAddress,
}
/*
impl<'a> NodeSlice<'a> {
    fn is_candidate_for_extraction(&self) -> bool {
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

#[derive(Debug, Clone)]
pub enum DeclarationType {
    Resolved(XcodeText),
    Unresolved(usize), // Index
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: XcodeText,
    pub var_type: DeclarationType, // Some types cannot be resolved in the first pass, and need to be queried from the LSP
    pub declared_in_node: NodeAddress,
    pub referenced_in_nodes: Vec<NodeAddress>,
}

fn is_child_of(parent: &NodeAddress, child: &NodeAddress) -> bool {
    for (i, el) in parent.iter().enumerate() {
        if child.get(i) != Some(&el) {
            return false;
        }
    }
    return true;
}

pub fn get_node_address(parent_address: &NodeAddress, node: &Node) -> NodeAddress {
    let mut result = parent_address.clone();
    result.push(node.id());
    result
}

#[cfg(test)]
mod tests {
    mod is_child_of {
        use crate::core_engine::features::complexity_refactoring::utils::is_child_of;

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
}
