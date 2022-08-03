use tree_sitter::{Node, Point, Tree};

use crate::core_engine::rules::{
    rule_match::RuleMatchProps, MatchRange, RuleBase, RuleMatch, RuleMatchCategory, RuleName,
    RuleResults, TextRange,
};

pub struct BracketHighlightProps {
    pub selected_text_range: Option<TextRange>,
    pub swift_syntax_tree: Option<Tree>,
    pub text_content: String,
}

pub struct BracketHighlightRule {
    rule_matches: Option<Vec<RuleMatch>>,
    rule_type: RuleName,
    selected_text_range: Option<TextRange>,
    swift_syntax_tree: Option<Tree>,
    text_content: String,
}

impl RuleBase for BracketHighlightRule {
    fn rule_type(&self) -> RuleName {
        self.rule_type.clone()
    }

    fn rule_matches(&self) -> Option<&Vec<RuleMatch>> {
        self.rule_matches.as_ref()
    }

    fn rule_results(&self) -> Option<RuleResults> {
        if let Some(rule_matches) = &self.rule_matches {
            Some(RuleResults {
                rule: self.rule_type(),
                results: rule_matches.clone(),
            })
        } else {
            None
        }
    }

    fn run(&mut self) -> Option<RuleResults> {
        println!("\n\n");
        let (selected_node, selected_text_range) = if let (Some(node), Some(selected_text_range)) = (
            self.get_selected_code_node(),
            self.selected_text_range.clone(),
        ) {
            (node, selected_text_range)
        } else {
            // Failed to get selected_node or selected_text_range
            return None;
        };
        println!(
            "SELECTED_NODE: {:?} Range: {:?}",
            selected_node,
            selected_node.range()
        );

        let code_block_node = if let Some(code_block_node) = get_code_block_parent(selected_node) {
            code_block_node
        } else {
            println!("Failed to get code_block_node");
            return None;
        };
        println!(
            "CODE_BLOCK_NODE: {:?} Range: {:?}",
            code_block_node,
            code_block_node.range()
        );

        // If cursor is touchin first or last character of range: also return parents bounds
        println!("SELECTED_TEXT_RANGE: {:?}", selected_text_range);

        let is_touching_left_first_char =
            selected_text_range.index == code_block_node.range().start_byte;
        let is_touching_right_first_char =
            selected_text_range.index == code_block_node.range().start_byte + 1;
        // Need to figure out how to check last character touch

        println!(
            "IS_TOUCHING_LEFT_FIRST_CHAR: {:?}",
            is_touching_left_first_char
        );

        let mut line_rule_matches = get_rule_matches_of_first_and_last_char_in_node(
            &code_block_node,
            RuleMatchCategory::BracketHighlightLine,
        );
        let mut touch_rule_matches: Vec<RuleMatch> = vec![];
        if is_touching_left_first_char || is_touching_right_first_char {
            touch_rule_matches = get_rule_matches_of_first_and_last_char_in_node(
                &code_block_node,
                RuleMatchCategory::BracketHighlightTouch,
            );
            // Get line bounds of parent
            if is_touching_left_first_char {
                println!("Get line bounds of parent");
                if let Some(code_block_parent_node) = get_code_block_parent(code_block_node) {
                    println!("code_block_parent_node {:?}", code_block_parent_node);
                    line_rule_matches = get_rule_matches_of_first_and_last_char_in_node(
                        &code_block_parent_node,
                        RuleMatchCategory::BracketHighlightLine,
                    );
                }
            }
        }

        self.rule_matches = Some(
            line_rule_matches
                .into_iter()
                .chain(touch_rule_matches)
                .collect(),
        );
        println!("RULE_MATCHES: {:?}", self.rule_matches);
        self.rule_results()
    }

    fn compute_rule_match_rectangles(&mut self, editor_app_pid: i32) -> Option<RuleResults> {
        if let Some(matches) = &mut self.rule_matches {
            for single_match in matches.iter_mut() {
                (*single_match).update_rectangles(editor_app_pid);
            }
        }

        self.rule_results()
    }
}

impl BracketHighlightRule {
    pub fn new(swift_syntax_tree: Option<Tree>) -> Self {
        Self {
            rule_matches: None,
            rule_type: RuleName::BracketHighlight,
            selected_text_range: None,
            swift_syntax_tree,
            text_content: "".to_string(),
        }
    }

    pub fn update_properties(&mut self, properties: BracketHighlightProps) {
        if properties.selected_text_range.is_some() {
            self.selected_text_range = properties.selected_text_range;
        }

        self.swift_syntax_tree = properties.swift_syntax_tree;
        self.text_content = properties.text_content;
    }

    fn get_selected_code_node(&self) -> Option<Node> {
        if let (Some(selected_text_range), Some(syntax_tree)) =
            (self.selected_text_range.clone(), &self.swift_syntax_tree)
        {
            if let Some((start_position, _)) =
                selected_text_range.as_StartEndTextPosition(&self.text_content)
            {
                let node = syntax_tree.root_node().named_descendant_for_point_range(
                    Point {
                        row: start_position.row,
                        column: start_position.column,
                    },
                    Point {
                        row: start_position.row,
                        column: start_position.column,
                    },
                );

                return node;
            }
        }
        None
    }
}

fn rule_match_from_range(range: TextRange, category: RuleMatchCategory) -> RuleMatch {
    RuleMatch::new(
        RuleName::BracketHighlight,
        MatchRange {
            string: "".to_string(),
            range,
        },
        RuleMatchProps {
            identifier: "".to_string(),
            description: "".to_string(),
            category,
        },
    )
}

fn get_code_block_parent(node_input: Node) -> Option<Node> {
    let code_block_kinds = vec![
        "value_arguments",
        "array_type",
        "array_literal",
        // "function_declaration",
        "function_body",
        // "class_declaration",
        "class_body",
        "if_statement",
        "guard_statement",
        "else_statement",
        "lambda_literal",
        "catch_block",
        "computed_property",
        "switch_statement",
        "switch_entry",
        "tuple_type",
        "while_statement",
    ];

    let mut node = node_input.clone();
    loop {
        if code_block_kinds.contains(&node.kind()) {
            return Some(node);
        }

        if let Some(parent) = node.parent() {
            node = parent;
        } else {
            return None;
        }
    }
}

fn get_rule_matches_of_first_and_last_char_in_node(
    node: &Node,
    category: RuleMatchCategory,
) -> Vec<RuleMatch> {
    let mut rule_matches: Vec<RuleMatch> = vec![];
    rule_matches.push(rule_match_from_range(
        TextRange {
            index: node.range().start_byte,
            length: 1,
        },
        category.clone(),
    ));
    rule_matches.push(rule_match_from_range(
        TextRange {
            index: node.range().end_byte,
            length: 1,
        },
        category.clone(),
    ));
    rule_matches
}

#[cfg(test)]
mod tests {}
