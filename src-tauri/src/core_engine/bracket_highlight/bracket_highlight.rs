use accessibility::AXUIElement;
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Point, Tree};
use ts_rs::TS;

use crate::{
    ax_interaction::get_textarea_uielement,
    core_engine::{
        ax_utils::calc_rectangles_and_line_matches,
        rules::{MatchRange, TextPosition, TextRange},
        types::MatchRectangle,
    },
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/bracket_highlight/")]
pub struct BracketHighlightBracket {
    text_range: TextRange,
    text_position: TextPosition,
    rectangle: MatchRectangle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/bracket_highlight/")]
pub struct BracketHighlightBracketPair {
    first: BracketHighlightBracket,
    last: BracketHighlightBracket,
}

impl BracketHighlightBracketPair {
    pub fn new(
        text_content: &String,
        first_range: TextRange,
        first_rectangle: MatchRectangle,
        last_range: TextRange,
        last_rectangle: MatchRectangle,
    ) -> Option<Self> {
        if let (Some(first_text_position), Some(last_text_position)) = (
            TextPosition::from_TextIndex(text_content, first_range.index),
            TextPosition::from_TextIndex(text_content, last_range.index),
        ) {
            Some(Self {
                first: BracketHighlightBracket {
                    text_range: first_range,
                    text_position: first_text_position,
                    rectangle: first_rectangle,
                },
                last: BracketHighlightBracket {
                    text_range: last_range,
                    text_position: last_text_position,
                    rectangle: last_rectangle,
                },
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/bracket_highlight/")]
pub struct BracketHighlightResults {
    lines: BracketHighlightBracketPair,
    lines_elbow_x: Option<usize>,
    boxes: BracketHighlightBracketPair,
}

pub struct BracketHighlight {
    results: Option<BracketHighlightResults>,
    selected_text_range: Option<TextRange>,
    swift_syntax_tree: Option<Tree>,
    text_content: Option<String>,
    window_pid: i32,
}

impl BracketHighlight {
    pub fn new(
        selected_text_range: Option<TextRange>,
        swift_syntax_tree: Option<Tree>,
        text_content: Option<String>,
        window_pid: i32,
    ) -> Self {
        Self {
            results: None,
            selected_text_range,
            swift_syntax_tree,
            text_content,
            window_pid,
        }
    }

    pub fn update_content(
        &mut self,
        swift_syntax_tree: Option<Tree>,
        text_content: Option<String>,
    ) {
        println!("update_content");
        self.swift_syntax_tree = swift_syntax_tree;
        self.text_content = text_content;
    }

    pub fn update_selected_text_range(&mut self, selected_text_range: Option<TextRange>) {
        println!("update_selected_text_range");
        self.selected_text_range = selected_text_range;
    }

    pub fn get_results(&self) -> Option<BracketHighlightResults> {
        self.results.clone()
    }

    pub fn generate_results(&mut self) {
        println!("generate_results");
        let (selected_node, selected_text_range, text_content, textarea_ui_element) = if let (
            Some(node),
            Some(selected_text_range),
            Some(text_content),
            Some(textarea_ui_element),
        ) = (
            self.get_selected_code_node(),
            self.selected_text_range.clone(),
            self.text_content.clone(),
            get_textarea_uielement(self.window_pid),
        ) {
            (node, selected_text_range, text_content, textarea_ui_element)
        } else {
            // Failed to get selected_node, selected_text_range, text_content, or ui_element
            self.results = None;
            return;
        };

        let code_block_node = if let Some(code_block_node) = get_code_block_parent(selected_node) {
            code_block_node
        } else {
            self.results = None;
            return;
        };

        let is_touching_left_first_char =
            selected_text_range.index == code_block_node.range().start_byte;
        // Need to figure out how to check last character touch

        let mut line_brackets_match_range =
            get_match_range_of_first_and_last_char_in_node(&code_block_node, &text_content);
        let box_brackets_match_range =
            get_match_range_of_first_and_last_char_in_node(&code_block_node, &text_content);
        // Get line bounds of parent
        if is_touching_left_first_char {
            if let Some(parent_node) = code_block_node.clone().parent() {
                if let Some(code_block_parent_node) = get_code_block_parent(parent_node) {
                    line_brackets_match_range = get_match_range_of_first_and_last_char_in_node(
                        &code_block_parent_node,
                        &text_content,
                    );
                }
            }
        }
        println!("line_brackets_match_range: {:?}", line_brackets_match_range);
        println!("box_brackets_match_range: {:?}", box_brackets_match_range);
        let (line_brackets_match_range, box_brackets_match_range) =
            if let (Some(line_brackets), Some(box_brackets)) =
                (line_brackets_match_range, box_brackets_match_range)
            {
                (line_brackets, box_brackets)
            } else {
                self.results = None;
                return;
            };

        // println!("line_brackets_match_range: {:?}", line_brackets_match_range);
        // println!("box_brackets_match_range: {:?}", box_brackets_match_range);

        // Get rectangles from the brackets
        let (first_line_rectangle, last_line_rectangle, first_box_rectangle, last_box_rectangle) =
            if let (
                Some(first_line_rectangle),
                Some(last_line_rectangle),
                Some(first_box_rectangle),
                Some(last_box_rectangle),
            ) = (
                rectangles_from_match_range(&line_brackets_match_range.0, &textarea_ui_element),
                rectangles_from_match_range(&line_brackets_match_range.1, &textarea_ui_element),
                rectangles_from_match_range(&box_brackets_match_range.0, &textarea_ui_element),
                rectangles_from_match_range(&box_brackets_match_range.1, &textarea_ui_element),
            ) {
                (
                    first_line_rectangle,
                    last_line_rectangle,
                    first_box_rectangle,
                    last_box_rectangle,
                )
            } else {
                self.results = None;
                return;
            };

        // println!("first_line_rectangle {:?}", first_line_rectangle);
        // println!("last_line_rectangle {:?}", last_line_rectangle);
        // println!("first_box_rectangle {:?}", first_box_rectangle);
        // println!("last_box_rectangle {:?}", last_box_rectangle);

        if let (Some(first_pair), Some(last_pair)) = (
            BracketHighlightBracketPair::new(
                &text_content,
                line_brackets_match_range.0.range,
                first_line_rectangle,
                line_brackets_match_range.1.range,
                last_line_rectangle,
            ),
            BracketHighlightBracketPair::new(
                &text_content,
                box_brackets_match_range.0.range,
                first_box_rectangle,
                box_brackets_match_range.1.range,
                last_box_rectangle,
            ),
        ) {
            self.results = Some(BracketHighlightResults {
                lines: first_pair,
                lines_elbow_x: None,
                boxes: last_pair,
            });
        } else {
            self.results = None;
        }
    }

    fn get_selected_code_node(&self) -> Option<Node> {
        if let (Some(selected_text_range), Some(syntax_tree), Some(text_content)) = (
            self.selected_text_range.clone(),
            &self.swift_syntax_tree,
            &self.text_content,
        ) {
            if let Some((start_position, _)) =
                selected_text_range.as_StartEndTextPosition(text_content)
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

fn rectangles_from_match_range(
    range: &MatchRange,
    textarea_ui_element: &AXUIElement,
) -> Option<MatchRectangle> {
    let (rectangles, _) = calc_rectangles_and_line_matches(range, &textarea_ui_element);
    if rectangles.len() == 1 {
        Some(rectangles[0].clone())
    } else {
        None
    }
}

fn get_code_block_parent(node_input: Node) -> Option<Node> {
    let code_block_kinds = vec![
        // "source_file",
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
        "do_statement",
        "catch_block",
        "computed_property",
        "switch_statement",
        "switch_entry",
        "tuple_type",
        "while_statement",
        "enum_class_body",
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

fn get_match_range_of_first_and_last_char_in_node(
    node: &Node,
    text: &String,
) -> Option<(MatchRange, MatchRange)> {
    let first = MatchRange::from_text_and_range(
        text,
        TextRange {
            index: node.range().start_byte,
            length: 1,
        },
    );
    let last = MatchRange::from_text_and_range(
        text,
        TextRange {
            index: node.range().end_byte - 1,
            length: 1,
        },
    );

    if let (Some(first), Some(last)) = (first, last) {
        Some((first, last))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {}
