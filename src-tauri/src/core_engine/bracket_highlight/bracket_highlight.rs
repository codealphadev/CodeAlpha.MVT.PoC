use accessibility::AXUIElement;
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Point, Tree};
use ts_rs::TS;

use crate::{
    ax_interaction::get_textarea_uielement,
    core_engine::{
        ax_utils::{calc_rectangles_and_line_matches, get_bounds_of_TextRange},
        rules::{get_index_of_next_row, MatchRange, TextPosition, TextRange},
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
    first: Option<BracketHighlightBracket>,
    last: Option<BracketHighlightBracket>,
}

impl BracketHighlightBracketPair {
    pub fn new(
        first_range: TextRange,
        first_rectangle: Option<MatchRectangle>,
        first_text_position: TextPosition,
        last_range: TextRange,
        last_rectangle: Option<MatchRectangle>,
        last_text_position: TextPosition,
    ) -> Self {
        let mut first = None;
        if let Some(first_rectangle) = first_rectangle {
            first = Some(BracketHighlightBracket {
                text_range: first_range,
                text_position: first_text_position,
                rectangle: first_rectangle,
            });
        }

        let mut last = None;
        if let Some(last_rectangle) = last_rectangle {
            last = Some(BracketHighlightBracket {
                text_range: last_range,
                text_position: last_text_position,
                rectangle: last_rectangle,
            });
        }

        Self { first, last }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/bracket_highlight/")]
pub struct BracketHighlightResults {
    lines: BracketHighlightBracketPair,
    lines_elbow_x: Option<f64>,
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
        self.swift_syntax_tree = swift_syntax_tree;
        self.text_content = text_content;
    }

    pub fn update_selected_text_range(&mut self, selected_text_range: Option<TextRange>) {
        self.selected_text_range = selected_text_range;
    }

    pub fn get_results(&self) -> Option<BracketHighlightResults> {
        self.results.clone()
    }

    pub fn generate_results(&mut self) {
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
        // println!("selected_node: {:?}", selected_node);

        // let (selected_text_position, _) = self
        //     .selected_text_range
        //     .unwrap()
        //     .as_StartEndTextPosition(&(self.text_content).clone().unwrap())
        //     .unwrap();
        // let tree_clone = (&self.swift_syntax_tree).clone().unwrap();
        // let unnamed_node = tree_clone.root_node().descendant_for_point_range(
        //     Point {
        //         row: selected_text_position.row,
        //         column: selected_text_position.column,
        //     },
        //     Point {
        //         row: selected_text_position.row,
        //         column: selected_text_position.column,
        //     },
        // );
        // println!("unnamed_node: {:?}", unnamed_node.unwrap());

        let code_block_node = if let Some(code_block_node) = get_code_block_parent(selected_node) {
            code_block_node
        } else {
            self.results = None;
            return;
        };
        // println!("code_block_node: {:?}", code_block_node);

        let mut line_brackets_match_range =
            get_match_range_of_first_and_last_char_in_node(&code_block_node, &text_content);
        let mut line_positions = (
            TextPosition::from_TSPoint(&code_block_node.start_position()),
            TextPosition::from_TSPoint(&code_block_node.end_position()),
        );
        let box_brackets_match_range = line_brackets_match_range.clone();
        let box_positions = line_positions.clone();

        // Get line bounds of parent
        let is_touching_left_first_char =
            selected_text_range.index == code_block_node.range().start_byte;

        if is_touching_left_first_char {
            if let Some(parent_node) = code_block_node.clone().parent() {
                if let Some(code_block_parent_node) = get_code_block_parent(parent_node) {
                    line_brackets_match_range = get_match_range_of_first_and_last_char_in_node(
                        &code_block_parent_node,
                        &text_content,
                    );
                    line_positions = (
                        TextPosition::from_TSPoint(&code_block_node.start_position()),
                        TextPosition::from_TSPoint(&code_block_node.end_position()),
                    );
                }
            }
        }

        let (line_brackets_match_range, box_brackets_match_range) =
            if let (Some(line_brackets), Some(box_brackets)) =
                (line_brackets_match_range, box_brackets_match_range)
            {
                (line_brackets, box_brackets)
            } else {
                self.results = None;
                return;
            };

        // Get rectangles from the match ranges

        let (first_line_rectangle, last_line_rectangle, first_box_rectangle, last_box_rectangle) = (
            rectangles_from_match_range(&line_brackets_match_range.0, &textarea_ui_element),
            rectangles_from_match_range(&line_brackets_match_range.1, &textarea_ui_element),
            rectangles_from_match_range(&box_brackets_match_range.0, &textarea_ui_element),
            rectangles_from_match_range(&box_brackets_match_range.1, &textarea_ui_element),
        );

        let line_pair = BracketHighlightBracketPair::new(
            line_brackets_match_range.0.range,
            first_line_rectangle,
            line_positions.clone().0,
            line_brackets_match_range.1.range,
            last_line_rectangle,
            line_positions.clone().1,
        );

        let box_pair = BracketHighlightBracketPair::new(
            box_brackets_match_range.0.range,
            first_box_rectangle,
            box_positions.0,
            box_brackets_match_range.1.range,
            last_box_rectangle,
            box_positions.1,
        );

        // Check if elbow is needed
        let mut lines_elbow_x = None;
        if (line_positions.0.row != line_positions.1.row) {
            let first_line_bracket_range = line_brackets_match_range.0.range.clone();
            if let Some(next_row_index) =
                get_index_of_next_row(first_line_bracket_range.index, &text_content)
            {
                if let Some(left_most_column) = get_left_most_column_in_rows(
                    TextRange {
                        index: next_row_index,
                        length: line_brackets_match_range.1.range.index - next_row_index,
                    },
                    &text_content,
                ) {
                    if let (Some(elbow_match_rectangle), Some(line_pair_last)) = (
                        get_bounds_of_TextRange(
                            &TextRange {
                                index: left_most_column.index,
                                length: 1,
                            },
                            &textarea_ui_element,
                        ),
                        line_pair.clone().last,
                    ) {
                        if line_pair_last.rectangle.origin.x > elbow_match_rectangle.origin.x {
                            lines_elbow_x = Some(elbow_match_rectangle.origin.x);
                        }
                        if let Some(first_line_rectangle) = first_line_rectangle {
                            if first_line_rectangle.origin.x < elbow_match_rectangle.origin.x {
                                lines_elbow_x = Some(first_line_rectangle.origin.x);
                            }
                        }
                    }
                }
            }
        }
        self.results = Some(BracketHighlightResults {
            lines: line_pair,
            lines_elbow_x,
            boxes: box_pair,
        });
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

#[derive(Debug, PartialEq)]
struct LeftMostColumn {
    index: usize,
    row: usize,
}

/// It takes a range of text and returns the index of the first non-whitespace character in the range
///
/// Arguments:
///
/// * `range`: TextRange - index should be first index of the lines that should be compared, last index should be end of code bracket
/// * `text_content`: The entire text content of the file.
///
/// Returns:
///
/// A LeftMostColumn struct
fn get_left_most_column_in_rows(range: TextRange, text_content: &String) -> Option<LeftMostColumn> {
    if text_content.len() < range.index + range.length {
        return None;
    }
    let text = &text_content[range.index..range.index + range.length];
    let mut left_most_column_option: Option<usize> = None;
    let mut left_most_row: usize = 0;
    let mut left_most_index: usize = 0;

    let mut index = range.index;
    let mut row = 0;

    for line in text.lines() {
        let mut column: usize = 0;
        for (_, c) in line.chars().enumerate() {
            if c != ' ' {
                break;
            }
            column += 1;
        }

        if line.chars().count() > 0 {
            if let Some(left_most_column) = left_most_column_option {
                if column < left_most_column {
                    left_most_column_option = Some(column);
                    left_most_row = row;
                    left_most_index = index + column;
                }
            } else {
                left_most_column_option = Some(column);
                left_most_row = row;
                left_most_index = index + column;
            }
        }

        index += line.len() + 1;
        row += 1;
    }

    if let Some(_) = left_most_column_option {
        Some(LeftMostColumn {
            index: left_most_index,
            row: left_most_row,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests_bracket_highlight {
    use crate::core_engine::rules::TextRange;

    use super::{get_left_most_column_in_rows, LeftMostColumn};

    #[test]
    fn test_get_left_most_column() {
        let text = "if (test) {
          print(x)
        }"
        .to_string();
        assert_eq!(
            get_left_most_column_in_rows(
                TextRange {
                    index: 12,
                    length: 27
                },
                &text
            ),
            Some(LeftMostColumn { index: 39, row: 1 })
        );

        let text_left_of_closing = "if (test) {
              print(x)
            print(y)

              }"
        .to_string();
        assert_eq!(
            get_left_most_column_in_rows(
                TextRange {
                    index: 12,
                    length: 58
                },
                &text_left_of_closing
            ),
            Some(LeftMostColumn { index: 47, row: 1 })
        );

        let text_on_last_row = "if (test) {
            print(x)
      print(y)}"
            .to_string();
        assert_eq!(
            get_left_most_column_in_rows(
                TextRange {
                    index: 12,
                    length: 35
                },
                &text_on_last_row
            ),
            Some(LeftMostColumn { index: 39, row: 1 })
        );

        let text_out_of_range = "if (test) { ".to_string();
        assert_eq!(
            get_left_most_column_in_rows(
                TextRange {
                    index: 12,
                    length: 35
                },
                &text_out_of_range
            ),
            None
        );
    }
}
