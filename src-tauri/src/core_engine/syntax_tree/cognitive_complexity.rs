use std::{collections::HashMap, ops};

use tree_sitter::Node;

#[derive(Debug, Clone)]
pub struct Complexities {
    nesting_complexity: isize,
    fundamental_complexity: isize,
}

impl Complexities {
    pub fn _complexity(&self) -> isize {
        self.nesting_complexity + self.fundamental_complexity
    }
}

impl ops::Add<Complexities> for Complexities {
    type Output = Complexities;
    fn add(self, rhs: Complexities) -> Complexities {
        Complexities {
            nesting_complexity: self.nesting_complexity + rhs.nesting_complexity,
            fundamental_complexity: self.fundamental_complexity + rhs.fundamental_complexity,
        }
    }
}
impl ops::AddAssign for Complexities {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            fundamental_complexity: self.fundamental_complexity + other.fundamental_complexity,
            nesting_complexity: self.nesting_complexity + other.nesting_complexity,
        }
    }
}

// Iterate through subtrees of a node and save an entry for the accumulated complexity of each node and its children, hashed by id in node_complexities
pub fn calculate_cognitive_complexities(
    node: &Node,
    mut nesting_depth: isize,
    output_complexities: &mut HashMap<usize, Complexities>,
) -> Complexities {
    let mut cursor = node.walk();
    let mut complexity: Complexities = Complexities {
        nesting_complexity: 0,
        fundamental_complexity: 0,
    };

    let mut cursor2 = node.walk();
    match cursor.node().kind() {
        "function_declaration" | "lambda_literal" => {
            nesting_depth += 1;
        }
        "if_statement" => {
            // else if should not increment nesting by 2; it leads to if_statement as direct parent of if_statement
            if let Some(parent) = node.parent() {
                if parent.kind() != "if_statement" {
                    complexity.nesting_complexity += (nesting_depth - 1).max(0);
                    complexity.fundamental_complexity += 1;

                    if (node
                        .children_by_field_name("condition", &mut cursor2)
                        .count())
                        > 1
                    {
                        complexity.fundamental_complexity += 1
                    }

                    nesting_depth += 1;
                }
            }
        }
        "for_statement"
        | "while_statement"
        | "repeat_while_statement"
        | "catch_block"
        | "switch_statement" => {
            complexity.nesting_complexity += (nesting_depth - 1).max(0);
            complexity.fundamental_complexity += 1;
            nesting_depth += 1;
        }
        "else" => {
            complexity.fundamental_complexity += 1;
        }
        "control_transfer_statement" => {
            if node.child_by_field_name("result").is_some() {
                complexity.fundamental_complexity += 1 // Penalize breaks with labels
            }
        }
        "conjunction_expression" => match node.parent().map(|p| p.kind()) {
            Some("conjunction_expression") => {}
            _ => {
                complexity.fundamental_complexity += 1;
            }
        },
        "disjunction_expression" => match node.parent().map(|p| p.kind()) {
            Some("disjunction_expression") => {}
            _ => {
                complexity.fundamental_complexity += 1;
            }
        },
        _ => {}
    }

    for child in cursor.node().named_children(&mut cursor) {
        if node.kind() == "if_statement" && child.kind() == "if_statement" {
            // else if should not increase nesting by 2. In case of else { if } there is a statements node in between
            complexity +=
                calculate_cognitive_complexities(&child, nesting_depth - 1, output_complexities);
        } else {
            complexity +=
                calculate_cognitive_complexities(&child, nesting_depth, output_complexities);
        }
    }
    output_complexities.insert(node.id(), complexity.clone());
    complexity
}
