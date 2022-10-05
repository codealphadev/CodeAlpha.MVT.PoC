pub type NodeAddress = Vec<usize>;

pub fn is_child_of(parent: &NodeAddress, child: &NodeAddress) -> bool {
    for (i, el) in parent.iter().enumerate() {
        if child.get(i) != Some(&el) {
            return false;
        }
    }
    return true;
}

pub fn get_node_address(parent_address: &NodeAddress, node_id: usize) -> NodeAddress {
    let mut result = parent_address.clone();
    result.push(node_id);
    result
}

#[cfg(test)]
mod tests {
    mod is_child_of {
        use crate::core_engine::features::complexity_refactoring::is_child_of;

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
