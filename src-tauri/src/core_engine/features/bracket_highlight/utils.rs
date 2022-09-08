use tree_sitter::Node;

use crate::{
    core_engine::{utils::XcodeText, TextPosition, TextRange},
    platform::macos::{get_bounds_for_TextRange, GetVia, XcodeError},
    utils::geometry::LogicalFrame,
};

use super::BracketHighlightError;

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

pub fn get_char_rectangle_from_text_index(
    index: usize,
) -> Result<Option<LogicalFrame>, BracketHighlightError> {
    match get_bounds_for_TextRange(&TextRange { index, length: 1 }, &GetVia::Current) {
        Ok(bounds) => Ok(Some(bounds)),
        Err(XcodeError::NotContainedVisibleTextRange) => Ok(None),
        Err(err) => Err(BracketHighlightError::GenericError(err.into())),
    }
}

pub fn length_to_code_block_body_start(
    node: &Node,
    text: &XcodeText,
    selected_text_index: usize,
) -> Result<(usize, bool), BracketHighlightError> {
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
                    return Ok((additional_index, is_selected_text_in_declaration));
                }
                additional_index += 1;
            }
        }
    }

    Err(BracketHighlightError::UnsupportedCodeblock)
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
        "function_declaration",
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

pub fn get_indexes_of_first_and_last_char_in_node(
    node: &Node,
    text: &XcodeText,
    selected_text_index: usize,
) -> Result<(usize, usize), BracketHighlightError> {
    if let (Some(mut first_index), Some(last_index)) = (
        get_node_start_index(&node, &text),
        get_node_end_index(&node, &text).map(|x| x - 1),
    ) {
        if let Ok(additional_length) =
            length_to_code_block_body_start(node, text, selected_text_index)
        {
            first_index += additional_length.0;
        }

        return Ok((first_index, last_index));
    }

    Err(BracketHighlightError::GenericError(anyhow::Error::msg(
        "Failed to get indexes of first and last char in node",
    )))
}

pub fn only_whitespace_on_line_until_position(
    position: TextPosition,
    text: &XcodeText,
) -> Result<bool, BracketHighlightError> {
    let rows = &text.rows;
    if position.row + 1 > rows.len() {
        return Err(BracketHighlightError::PositionOutOfBounds);
    }

    let row = &rows[position.row];
    if row.len() == 0 {
        // e.g. last line in document
        return Ok(true);
    }
    if position.column + 1 > row.len() {
        // remove + 1?
        return Err(BracketHighlightError::PositionOutOfBounds);
    }

    for c_u16 in row[0..position.column].into_iter() {
        if !XcodeText::char_is_whitespace(c_u16) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Debug, PartialEq)]
pub struct IndexAndRow {
    pub index: usize,
    pub row: usize,
}

pub fn get_text_index_of_left_most_char_in_range(
    range: TextRange,
    text: &XcodeText,
) -> Option<usize> {
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
        let (_, index, non_whitespace_column_i) = rows_data[0];
        return Some(index + non_whitespace_column_i);
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
            assert_eq!(result.ok(), expected);
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
            assert_eq!(result.is_ok(), false);
        }

        #[test]
        fn empty_column() {
            test_fn("", 0, 1, Some(true));
        }
    }

    #[cfg(test)]
    mod get_left_most_column_in_rows {
        use crate::core_engine::{
            features::bracket_highlight::utils::get_text_index_of_left_most_char_in_range,
            utils::XcodeText, TextRange,
        };

        fn test_fn(text: &str, index: usize, length: usize, expected: Option<usize>) {
            assert_eq!(
                get_text_index_of_left_most_char_in_range(
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
                Some(44),
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
                Some(55),
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
                Some(51),
            );
        }

        #[test]
        fn empty_lines() {
            test_fn(
                "self.init(


                  forKnownProcessID: app.processIdentifier)",
                11,
                61,
                Some(31),
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
