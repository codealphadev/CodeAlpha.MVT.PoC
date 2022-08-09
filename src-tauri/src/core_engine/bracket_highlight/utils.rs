use accessibility::AXUIElement;
use tree_sitter::Node;

use crate::core_engine::{
    ax_utils::{calc_rectangles_and_line_matches, get_text_range_of_line, is_text_of_line_wrapped},
    rules::{TextPosition, TextRange},
    types::{MatchRange, MatchRectangle},
};

fn bad_code_block_kinds() -> Vec<&'static str> {
    vec![
        "do_statement",
        "else_statement",
        "for_statement",
        "guard_statement",
        "if_statement",
        // "switch_entry", // case uses : instead of {
        "switch_statement",
        "while_statement",
    ]
}

pub fn rectangles_from_match_range(
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

pub fn length_to_bad_code_block_start(
    node: &Node,
    text_content: &String,
    selected_text_index: usize,
) -> Option<(usize, bool)> {
    let mut is_selected_text_in_bad_declaration = false;
    if bad_code_block_kinds().contains(&node.kind()) {
        let text_from_index = &text_content[node.range().start_byte..node.range().end_byte];
        let mut additional_index: usize = 0;
        for c in text_from_index.chars() {
            if c == '{' {
                if selected_text_index < node.range().start_byte + additional_index
                    && selected_text_index >= node.range().start_byte
                {
                    is_selected_text_in_bad_declaration = true;
                }
                return Some((additional_index, is_selected_text_in_bad_declaration));
            }
            additional_index += 1;
        }
    }
    None
}

pub fn get_code_block_parent(node_input: Node, ignore_current_bad_node: bool) -> Option<Node> {
    let code_block_kinds = vec![
        "array_literal",
        "array_type",
        "catch_block",
        "class_body",
        "computed_property",
        "do_statement",
        "else_statement",
        "enum_class_body",
        "for_statement",
        "function_body",
        "guard_statement",
        "if_statement",
        "lambda_literal",
        "switch_entry",
        "switch_statement",
        "tuple_type",
        "value_arguments",
        "while_statement",
        // "class_declaration",
        // "function_declaration",
        // "source_file",
    ];

    let mut node = node_input.clone();
    let mut parent_node = None;

    if ignore_current_bad_node && bad_code_block_kinds().contains(&node.kind()) {
        if let Some(parent) = node.parent() {
            node = parent;
        }
    }

    loop {
        if code_block_kinds.contains(&node.kind()) {
            parent_node = Some(node);
            break;
        }

        if let Some(parent) = node.parent() {
            node = parent;
        } else {
            break;
        }
    }
    parent_node
}

pub fn get_match_range_of_first_and_last_char_in_node(
    node: &Node,
    text: &String,
    selected_text_index: usize,
) -> Option<(MatchRange, MatchRange)> {
    let mut first_option = MatchRange::from_text_and_range(
        text,
        TextRange {
            index: node.range().start_byte,
            length: 1,
        },
    );
    let last_option = MatchRange::from_text_and_range(
        text,
        TextRange {
            index: node.range().end_byte - 1,
            length: 1,
        },
    );
    if let Some(additional_length) = length_to_bad_code_block_start(node, text, selected_text_index)
    {
        first_option = MatchRange::from_text_and_range(
            text,
            TextRange {
                index: node.range().start_byte + additional_length.0,
                length: 1,
            },
        );
    }

    if let (Some(first), Some(last)) = (first_option, last_option) {
        Some((first, last))
    } else {
        None
    }
}

pub fn rectanges_of_wrapped_line(
    row: usize,
    content: &String,
    textarea_ui_element: AXUIElement,
) -> Vec<MatchRectangle> {
    if let Some(is_wrapped) = is_text_of_line_wrapped(row, &textarea_ui_element) {
        if is_wrapped.0 {
            // line is wrapped
            if let Some(text_range) = get_text_range_of_line(row, &textarea_ui_element) {
                if let Some(match_range) = MatchRange::from_text_and_range(&content, text_range) {
                    let (rectangles, _) =
                        calc_rectangles_and_line_matches(&match_range, &textarea_ui_element);
                    return rectangles;
                }
            }
        }
    }

    vec![]
}

pub fn only_whitespace_on_line_until_position(position: TextPosition, text: &String) -> bool {
    let text_clone = text.clone();
    let row = &text_clone.lines().collect::<Vec<&str>>()[position.row][0..position.column];
    for c in row.chars() {
        if c != ' ' {
            return false;
        }
    }
    true
}

#[derive(Debug, PartialEq)]
pub struct LeftMostColumn {
    pub index: usize,
    pub row: usize,
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
pub fn get_left_most_column_in_rows(
    range: TextRange,
    text_content: &String,
) -> Option<LeftMostColumn> {
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

        if line.trim().chars().count() > 0 {
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
    use crate::core_engine::rules::{TextPosition, TextRange};

    use super::only_whitespace_on_line_until_position;

    #[test]
    fn test_only_whitespace_on_line_until_position() {
        let text = "if (test) {
        print(x)
       }"
        .to_string();
        assert_eq!(
            only_whitespace_on_line_until_position(TextPosition { row: 2, column: 9 }, &text),
            true
        );

        let text_false = "if (test) {
        print(x)
       ss}"
        .to_string();
        assert_eq!(
            only_whitespace_on_line_until_position(
                TextPosition { row: 2, column: 11 },
                &text_false
            ),
            false
        );

        let text_false = "if (test) { }".to_string();
        assert_eq!(
            only_whitespace_on_line_until_position(
                TextPosition { row: 0, column: 12 },
                &text_false
            ),
            false
        );
    }

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
                    length: 29
                },
                &text
            ),
            Some(LeftMostColumn { index: 40, row: 1 })
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
                    length: 60
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
                    length: 36
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

        let text_out_of_range = "self.init(
      
              
            forKnownProcessID: app.processIdentifier)"
            .to_string();
        assert_eq!(
            get_left_most_column_in_rows(
                TextRange {
                    index: 11,
                    length: 81
                },
                &text_out_of_range
            ),
            Some(LeftMostColumn { index: 51, row: 2 })
        );
    }
}
