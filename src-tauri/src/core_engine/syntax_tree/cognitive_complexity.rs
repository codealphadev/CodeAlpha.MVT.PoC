use std::{collections::HashMap, ops};

use crate::core_engine::XcodeText;
use tree_sitter::Node;

use super::{get_node_text, swift_syntax_tree::NodeMetadata, SwiftCodeBlockError};

#[derive(Debug, Clone, PartialEq)]
pub struct Complexities {
    nesting_complexity: isize,
    fundamental_complexity: isize,
}

impl Complexities {
    pub fn get_total_complexity(&self) -> isize {
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

pub fn calculate_cognitive_complexities(
    node: &Node,
    text_content: &XcodeText,
    output_node_metadata: &mut HashMap<usize, NodeMetadata>,
) -> Result<Complexities, SwiftCodeBlockError> {
    calculate_cognitive_complexities_intl(&node, &text_content, output_node_metadata, 0, vec![])
}

// Iterate through subtrees of a node and save an entry for the accumulated complexity of each node and its children, hashed by id in node_complexities
fn calculate_cognitive_complexities_intl(
    node: &Node,
    text_content: &XcodeText,
    output_node_metadata: &mut HashMap<usize, NodeMetadata>,
    mut nesting_depth: isize,
    mut parent_function_names: Vec<XcodeText>,
) -> Result<Complexities, SwiftCodeBlockError> {
    let mut complexity: Complexities = Complexities {
        nesting_complexity: 0,
        fundamental_complexity: 0,
    };

    match node.kind() {
        "function_declaration" | "lambda_literal" => {
            nesting_depth += 1;
            if let Some(name) = get_function_name(node, text_content) {
                parent_function_names.push(name);
            }
        }
        "ternary_expression" => {
            complexity.nesting_complexity += (nesting_depth - 1).max(0);
            complexity.fundamental_complexity += 1;
            nesting_depth += 1;
        }
        "if_statement" => {
            // else if should not increment nesting by 2; it leads to if_statement as direct parent of if_statement
            if let Some(parent) = node.parent() {
                if parent.kind() != "if_statement" {
                    complexity.nesting_complexity += (nesting_depth - 1).max(0);
                    complexity.fundamental_complexity += 1;

                    if (node
                        .children_by_field_name("condition", &mut node.walk())
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
        | "guard_statement"
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
            if control_transfer_statement_is_penalizable(node) {
                complexity.fundamental_complexity += 1
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
        "call_expression" => {
            // Check for recursive function call (only works if whole tree was parsed)
            if let Some(first_child) = node.child(0) {
                if first_child.kind() == "simple_identifier"
                    && parent_function_names.contains(&get_node_text(&first_child, text_content)?)
                {
                    complexity.fundamental_complexity += 1;
                }
            }
        }
        _ => {}
    }

    for child in node.named_children(&mut node.walk()) {
        if node.kind() == "if_statement" && child.kind() == "if_statement" {
            // else if should not increase nesting by 2. In case of else { if } there is a statements node in between
            complexity += calculate_cognitive_complexities_intl(
                &child,
                &text_content,
                output_node_metadata,
                nesting_depth - 1,
                parent_function_names.clone(),
            )?;
        } else {
            complexity += calculate_cognitive_complexities_intl(
                &child,
                &text_content,
                output_node_metadata,
                nesting_depth,
                parent_function_names.clone(),
            )?;
        }
    }
    output_node_metadata.insert(
        node.id(),
        NodeMetadata {
            complexities: complexity.clone(),
        },
    );
    Ok(complexity)
}

// TODO: Use Result instead of Option
fn get_function_name(node: &Node, text_content: &XcodeText) -> Option<XcodeText> {
    let x = node.child_by_field_name("name")?;
    get_node_text(&x, &text_content).ok()
}

fn control_transfer_statement_is_penalizable(control_transfer_statement: &Node) -> bool {
    // Penalize break <label> and continue <label>, but not throws, returns, or other control transfer statements
    for child in control_transfer_statement.children(&mut control_transfer_statement.walk()) {
        if child.kind() == "break" || child.kind() == "continue" {
            if control_transfer_statement
                .child_by_field_name("result")
                .is_some()
            {
                return true;
            }
        }
    }
    return false;
}

#[cfg(test)]
mod tests {
    mod calculate_cognitive_complexities {
        use std::collections::HashMap;

        use tree_sitter::Parser;

        use crate::core_engine::{
            syntax_tree::{
                calculate_cognitive_complexities, swift_syntax_tree::NodeMetadata, Complexities,
            },
            XcodeText,
        };

        #[test]
        fn correctly_calculates_complexities() {
            let text_content = XcodeText::from_str(
                r#"
                func addAnimatifsdfasdfasdfadsfasdfasdfdsons(
                    param: String,
                ) {
                    switch event.charactersIgnoringModifiers! {                 // +1 for switch
                        case "x":
                            if NSApp.sendAction(#selector(NSText.cut(_:)), to: nil, from: self) { return true }       // +1 for if, +1 for nesting
                        case "c":
                            if NSApp.sendAction(#selector(NSText.copy(_:)), to: nil, from: self) { return true }      // +1 for if, +1 for nesting
                        case "v":
                            if NSApp.sendAction(#selector(NSText.paste(_:)), to: nil, from: self) { return true }     // +1 for if, +1 for nesting
                        case "z":
                            if NSApp.sendAction(Selector(("undo:")), to: nil, from: self) { return true }             // +1 for if, +1 for nesting
                        case "a":
                            if NSApp.sendAction(#selector(NSResponder.selectAll(_:)), to: nil, from: self) { return true }  // +1 for if, +1 for nesting
                        default:
                            break
                        }
                    }
                    
                }
            "#,
            );

            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_swift::language())
                .expect("Swift Language not found");
            let tree = parser.parse_utf16(text_content.clone(), None).unwrap();
            let expected_complexity: Complexities = Complexities {
                nesting_complexity: 5,
                fundamental_complexity: 6,
            };
            let mut node_metadata = HashMap::<usize, NodeMetadata>::new();
            let calculated_complexity = calculate_cognitive_complexities(
                &tree.root_node(),
                &text_content,
                &mut node_metadata,
            );
            assert_eq!(expected_complexity, calculated_complexity.unwrap());
            assert_eq!(
                expected_complexity,
                node_metadata
                    .get(&tree.root_node().id())
                    .unwrap()
                    .complexities
            );
        }

        #[test]
        fn repeating_conjunction_disjunction_patterns() {
            let text_content = XcodeText::from_str(
                r#"
                func addAnimatifsdfasdfasdfadsfasdfasdfdsons(
                    param: String,
                ) {
                    if (a                                // +1 for if
                        && b && c && g                   // +1
                        || d || e                        // +1
                        &&                               // +1 
                        !(f && h)                        // +1     
                    ) {                          
                        return true;
                    }
                }
            "#,
            );

            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_swift::language())
                .expect("Swift Language not found");
            let tree = parser.parse_utf16(text_content.clone(), None).unwrap();
            let expected_complexity: Complexities = Complexities {
                nesting_complexity: 0,
                fundamental_complexity: 5,
            };
            let mut node_metadata = HashMap::<usize, NodeMetadata>::new();
            let calculated_complexity = calculate_cognitive_complexities(
                &tree.root_node(),
                &text_content,
                &mut node_metadata,
            );
            assert_eq!(expected_complexity, calculated_complexity.unwrap());
        }

        #[test]
        fn recursion() {
            let text_content = XcodeText::from_str(
                r#"
                func fibonacciRecursiveNum1(num1: Int, num2: Int, steps: Int) {
                    if steps > 0 {                                                   // +1 for if 
                        let newNum = num1 + num2
                        fibonacciRecursiveNum1(num2, num2: newNum, steps: steps-1)   // +1 for recursion
                    }
                    else {                                                           // +1
                        print("result = \(num2)")
                    }
                }
            "#,
            );

            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_swift::language())
                .expect("Swift Language not found");
            let tree = parser.parse_utf16(text_content.clone(), None).unwrap();
            let expected_complexity: Complexities = Complexities {
                nesting_complexity: 0,
                fundamental_complexity: 3,
            };
            let mut node_metadata = HashMap::<usize, NodeMetadata>::new();
            let calculated_complexity = calculate_cognitive_complexities(
                &tree.root_node(),
                &text_content,
                &mut node_metadata,
            );
            assert_eq!(expected_complexity, calculated_complexity.unwrap());
        }

        #[test]
        fn deep_recursion() {
            let text_content = XcodeText::from_str(
                r#"
                func recursive() {
                    func b() {
                        func c() {
                            b();            // +1 for recursion
                            recursive();    // +1 for recursion
                        }
                    }
                }
            "#,
            );

            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_swift::language())
                .expect("Swift Language not found");
            let tree = parser.parse_utf16(text_content.clone(), None).unwrap();
            let expected_complexity: Complexities = Complexities {
                nesting_complexity: 0,
                fundamental_complexity: 2,
            };
            let mut node_metadata = HashMap::<usize, NodeMetadata>::new();
            let calculated_complexity = calculate_cognitive_complexities(
                &tree.root_node(),
                &text_content,
                &mut node_metadata,
            );
            assert_eq!(expected_complexity, calculated_complexity.unwrap());
        }

        #[test]
        fn nesting_with_closures() {
            let text_content = XcodeText::from_str(
                r#"
                func fn() {
                    a("param1", { (no1, no2) in
                        if b > 4 {                  // +1 for if, +1 for nesting
                            return no1 + no2
                        } 
                    })
                }
            "#,
            );

            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_swift::language())
                .expect("Swift Language not found");
            let tree = parser.parse_utf16(text_content.clone(), None).unwrap();
            let expected_complexity: Complexities = Complexities {
                nesting_complexity: 1,
                fundamental_complexity: 1,
            };
            let mut node_metadata = HashMap::<usize, NodeMetadata>::new();
            let calculated_complexity = calculate_cognitive_complexities(
                &tree.root_node(),
                &text_content,
                &mut node_metadata,
            );
            assert_eq!(expected_complexity, calculated_complexity.unwrap());
        }
        #[test]
        fn control_flow_changes() {
            let text_content = XcodeText::from_str(
                r#"
                    func a() {
                        break label;     // +1 for break label
                        continue label;  // +1 for continue label
                    }
                "#,
            );

            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_swift::language())
                .expect("Swift Language not found");
            let tree = parser.parse_utf16(text_content.clone(), None).unwrap();
            let expected_complexity: Complexities = Complexities {
                nesting_complexity: 0,
                fundamental_complexity: 2,
            };
            let mut node_metadata = HashMap::<usize, NodeMetadata>::new();
            let calculated_complexity = calculate_cognitive_complexities(
                &tree.root_node(),
                &text_content,
                &mut node_metadata,
            );
            assert_eq!(expected_complexity, calculated_complexity.unwrap());
        }
    }
}
