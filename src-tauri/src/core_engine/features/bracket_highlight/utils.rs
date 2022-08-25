use accessibility::AXUIElement;
use tree_sitter::Node;

use crate::core_engine::{
    rules::{
        calc_rectangles_and_line_matches, get_text_range_of_line, is_text_of_line_wrapped,
        MatchRange, MatchRectangle,
    },
    utils::XcodeText,
    TextPosition, TextRange,
};

fn code_block_kinds_with_declaration() -> Vec<&'static str> {
    vec![
        "catch_block",
        "do_statement",
        "else_statement",
        "for_statement",
        "guard_statement",
        "if_statement",
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

pub fn length_to_code_block_body_start(
    node: &Node,
    text: &XcodeText,
    selected_text_index: usize,
) -> Option<(usize, bool)> {
    let mut is_selected_text_in_declaration = false;
    if code_block_kinds_with_declaration().contains(&node.kind()) {
        if let (Some(first_index), Some(last_index)) = (
            get_node_start_index(&node, &text),
            get_node_end_index(&node, &text),
        ) {
            let text_from_index = &text[first_index..last_index];
            let mut additional_index: usize = 0;
            for c in text_from_index {
                if *c == '{' as u16 {
                    if selected_text_index < first_index + additional_index
                        && selected_text_index >= first_index
                    {
                        is_selected_text_in_declaration = true;
                    }
                    return Some((additional_index, is_selected_text_in_declaration));
                }
                additional_index += 1;
            }
        }
    }
    None
}

pub fn get_code_block_parent(node_input: Node, ignore_declaration: bool) -> Option<Node> {
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
        // "switch_entry", // 'case' should not be highlighted
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

    if ignore_declaration && code_block_kinds_with_declaration().contains(&node.kind()) {
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
    text: &XcodeText,
    selected_text_index: usize,
) -> Option<(MatchRange, MatchRange)> {
    if let (Some(first_index), Some(last_index)) = (
        get_node_start_index(&node, &text),
        get_node_end_index(&node, &text),
    ) {
        let mut first_option = MatchRange::from_text_and_range(
            text,
            TextRange {
                index: first_index,
                length: 1,
            },
        );
        let last_option = MatchRange::from_text_and_range(
            text,
            TextRange {
                index: last_index - 1,
                length: 1,
            },
        );

        if let Some(additional_length) =
            length_to_code_block_body_start(node, text, selected_text_index)
        {
            first_option = MatchRange::from_text_and_range(
                text,
                TextRange {
                    index: first_index + additional_length.0,
                    length: 1,
                },
            );
        }

        if let (Some(first), Some(last)) = (first_option, last_option) {
            return Some((first, last));
        }
    }
    None
}

pub fn rectanges_of_wrapped_line(
    row: usize,
    content: &XcodeText,
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

pub fn only_whitespace_on_line_until_position(
    position: TextPosition,
    text: &XcodeText,
) -> Option<bool> {
    let rows = &text.rows;
    if rows.len() == 0 {
        return None;
    }

    if rows.len() - 1 < position.row {
        return None;
    }

    let row = &rows[position.row];
    if row.len() - 1 < position.column {
        return None;
    }

    for c_u16 in row[0..position.column].into_iter() {
        if !XcodeText::char_is_whitespace(c_u16) {
            return Some(false);
        }
    }
    Some(true)
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
pub fn get_left_most_column_in_rows(range: TextRange, text: &XcodeText) -> Option<LeftMostColumn> {
    if text.len() < range.index + range.length {
        return None;
    }
    let text = XcodeText::from_array(&text[range.index..range.index + range.length]);
    let mut index = range.index;
    let mut rows_data = vec![];

    for (row_i, row) in text.rows_iter().enumerate() {
        if let Some(non_whitespace_column_i) =
            row.iter().position(|c| !XcodeText::char_is_whitespace(c))
        {
            rows_data.push((row_i, index, non_whitespace_column_i));
        }
        index += row.len() + 1;
    }
    rows_data.sort_by(|a, b| a.2.cmp(&b.2));

    if rows_data.len() > 0 {
        let (row_i, index, non_whitespace_column_i) = rows_data[0];
        return Some(LeftMostColumn {
            index: index + non_whitespace_column_i,
            row: row_i,
        });
    }
    None
}

fn get_node_start_index(node: &Node, text: &XcodeText) -> Option<usize> {
    TextPosition::from_TSPoint(&node.start_position()).as_TextIndex(&text)
}

fn get_node_end_index(node: &Node, text: &XcodeText) -> Option<usize> {
    TextPosition::from_TSPoint(&node.end_position()).as_TextIndex(&text)
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod only_whitespace_on_line_until_position {
        use crate::core_engine::{
            features::bracket_highlight::utils::only_whitespace_on_line_until_position,
            utils::XcodeText, TextPosition,
        };

        fn test_fn(text: &str, row: usize, column: usize, expected: Option<bool>) {
            let text = XcodeText::from_str(text);
            let result =
                only_whitespace_on_line_until_position(TextPosition { row, column }, &text);
            assert_eq!(result, expected);
        }

        #[test]
        fn last_row() {
            test_fn(
                "if (test) {
            print(x)
          }",
                2,
                10,
                Some(true),
            );
        }

        #[test]
        fn last_row_false() {
            test_fn(
                "if (test) {
     print(x)
         x}",
                2,
                10,
                Some(false),
            );
        }

        #[test]
        fn middle_row() {
            test_fn(
                "if (test) {
            print(x)
          }",
                1,
                12,
                Some(true),
            );
        }

        #[test]
        fn middle_row_false() {
            test_fn(
                "if (test) {
      x     print(x)
          }",
                1,
                12,
                Some(false),
            );
        }

        #[test]
        fn out_of_bounds_row() {
            test_fn(
                "if (test) {
            print(x)
          }",
                3,
                0,
                None,
            );
        }

        #[test]
        fn out_of_bounds_col() {
            test_fn(
                "if (test) {
            print(x)
          }",
                12,
                0,
                None,
            );
        }

        #[test]
        fn empty_text() {
            let result = only_whitespace_on_line_until_position(
                TextPosition { row: 10, column: 5 },
                &XcodeText::new_empty(),
            );
            assert_eq!(result, None);
        }
    }

    #[cfg(test)]
    mod get_left_most_column_in_rows {
        use crate::core_engine::{
            features::bracket_highlight::utils::{get_left_most_column_in_rows, LeftMostColumn},
            utils::XcodeText,
            TextRange,
        };

        fn test_fn(text: &str, index: usize, length: usize, expected: Option<LeftMostColumn>) {
            assert_eq!(
                get_left_most_column_in_rows(
                    TextRange { index, length },
                    &XcodeText::from_str(text)
                ),
                expected
            );
        }

        #[test]
        fn last_row() {
            test_fn(
                "if (test) {
            print(x)
           }",
                12,
                33,
                Some(LeftMostColumn { index: 44, row: 1 }),
            );
        }

        #[test]
        fn middle_row() {
            test_fn(
                "if (test) {
                  print(x)
                print(y)
      
                  }",
                12,
                78,
                Some(LeftMostColumn { index: 55, row: 1 }),
            );
        }

        #[test]
        fn text_on_last_row() {
            test_fn(
                "if (test) {
                  print(x)
            print(y)}",
                12,
                48,
                Some(LeftMostColumn { index: 51, row: 1 }),
            );
        }

        #[test]
        fn empty_lines() {
            test_fn(
                "self.init(


                  forKnownProcessID: app.processIdentifier)",
                11,
                61,
                Some(LeftMostColumn { index: 31, row: 2 }),
            );
        }

        #[test]
        fn out_of_range() {
            test_fn(
                "self.init(


                  forKnownProcessID: app.processIdentifier)",
                11,
                62,
                None,
            );
        }
    }
}
