#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A position in a multi-line text document, in terms of rows and columns.
/// Rows and columns are zero-based.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct TextPosition {
    pub row: usize,
    pub column: usize,
}

impl TextPosition {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }

    pub fn from_TSPoint(tree_sitter_point: &tree_sitter::Point) -> Self {
        Self {
            row: tree_sitter_point.row,
            column: tree_sitter_point.column,
        }
    }

    /// > Given a string and an index, return the row number and column number of the character at that
    /// index. Different from TextPosition, index does include the newline character.
    /// In case the index references a new line character in the text, we return the position of the
    /// next valid character.
    ///
    /// Arguments:
    ///
    /// * `text`: The text to search through.
    /// * `index`: The index of the character in the text.
    ///
    /// Returns:
    ///
    /// A TextPosition struct
    pub fn from_TextIndex(text: &String, index: usize) -> Option<TextPosition> {
        let mut row = 0;
        let mut col = 0;
        let mut text_iter = text.char_indices();
        while let Some((text_index, char)) = text_iter.next() {
            if text_index == index {
                return Some(TextPosition {
                    row: row,
                    column: col,
                });
            }

            if char == '\n' {
                row += 1;
                col = 0;
                continue;
            }

            col += 1;
        }

        None
    }

    pub fn as_TSPoint(&self) -> tree_sitter::Point {
        tree_sitter::Point {
            row: self.row,
            column: self.column,
        }
    }

    pub fn as_TextIndex(&self, text: &String) -> Option<usize> {
        self.as_TextIndex_stay_on_line(text, false)
    }

    pub fn as_TextIndex_stay_on_line(&self, text: &String, stay_on_line: bool) -> Option<usize> {
        let mut row = 0;
        let mut col = 0;
        let mut text_iter = text.char_indices();
        while let Some((text_index, char)) = text_iter.next() {
            if self.row == row && self.column == col {
                return Some(text_index);
            }

            if char == '\n' {
                // if stay_on_line is true, we want to return the index of the last character of the line self.row
                if stay_on_line && self.row == row {
                    return Some(text_index.clone());
                }
                row += 1;
                col = 0;
                continue;
            }

            col += 1;
        }

        None
    }
}

#[cfg(test)]
mod tests_TextPosition {
    use crate::core_engine::rules::utils::text_types::TextPosition;

    #[test]
    fn test_TextPosition_from_TextIndex_respects_new_line_character() {
        let text = "\nHello, \nWorld!";
        let index = 12; // ... starting from zero, so the 13th character, which is the 'l'
        assert_eq!(text.chars().nth(index).unwrap(), 'l');

        let position_option = TextPosition::from_TextIndex(&text.to_string(), index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 2);
        assert_eq!(position.column, 3);
    }

    #[test]
    fn test_TextPosition_from_TextIndex_one_line() {
        let text = "Hello, World!";
        let index = 5;
        let position_option = TextPosition::from_TextIndex(&text.to_string(), index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 0);
        assert_eq!(position.column, 5);
    }

    #[test]
    fn test_TextPosition_from_TextIndex_two_lines() {
        let text = "Hello, World!\nGoodbye, World!";
        let index = 20; // ... starting from zero, so the 21th character, which is the 'e'
        assert_eq!(text.chars().nth(index).unwrap(), 'e');
        let position_option = TextPosition::from_TextIndex(&text.to_string(), index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 1);
        assert_eq!(position.column, 6);
    }

    #[test]
    fn test_TextPosition_from_TextIndex_too_far() {
        let text = "Hello, World!";
        let index = 100;
        let position_option = TextPosition::from_TextIndex(&text.to_string(), index);

        assert_eq!(position_option.is_none(), true);
    }

    #[test]
    fn test_convert_TextPosition_as_TextIndex() {
        let text = "Hello, World!";
        let position = TextPosition::new(0, 5);
        let index_option = position.as_TextIndex(&text.to_string());

        assert_eq!(index_option.is_some(), true);

        let index = index_option.unwrap();

        assert_eq!(index, 5);
    }

    #[test]
    fn test_convert_TextPosition_as_TextIndex_multi_line() {
        let text = "Hello,\n World\n!";
        let position = TextPosition::new(2, 0);
        let index_option = position.as_TextIndex(&text.to_string());

        assert_eq!(index_option.is_some(), true);

        let index = index_option.unwrap();

        assert_eq!(index, 14);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far() {
        let text = "Hello, World!";
        let position = TextPosition::new(0, 100);
        let index_option = position.as_TextIndex(&text.to_string());

        assert_eq!(index_option.is_none(), true);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far_multiline_stay_on_line() {
        let text = "Hello,\nWorld!\n";
        let position = TextPosition::new(0, 100);
        let index_option = position.as_TextIndex_stay_on_line(&text.to_string(), true);

        assert_eq!(index_option.unwrap(), 6);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far_multiline_stay_on_line_false() {
        let text = "Hello,\nWorld!\n";
        let position = TextPosition::new(0, 100);
        let index_option = position.as_TextIndex_stay_on_line(&text.to_string(), false);

        assert_eq!(index_option.is_none(), true);
    }
}

/// A position in a multi-line text document, in terms of index and length.
/// Index is zero-based.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct TextRange {
    pub index: usize,
    pub length: usize,
}

impl TextRange {
    pub fn new(index: usize, length: usize) -> Self {
        Self { index, length }
    }

    pub fn from_StartEndIndex(start_index: usize, end_index: usize) -> TextRange {
        TextRange {
            index: start_index,
            length: end_index - start_index + 1,
        }
    }

    pub fn from_StartEndTextPosition(
        text: &String,
        start_position: &TextPosition,
        end_position: &TextPosition,
    ) -> Option<TextRange> {
        if let (Some(start_index), Some(end_index)) = (
            start_position.as_TextIndex(text),
            end_position.as_TextIndex(text),
        ) {
            return Some(TextRange::from_StartEndIndex(start_index, end_index));
        } else {
            return None;
        }
    }

    pub fn from_StartEndTSPoint(
        text: &String,
        start_position: &tree_sitter::Point,
        end_position: &tree_sitter::Point,
    ) -> Option<TextRange> {
        Self::from_StartEndTextPosition(
            text,
            &TextPosition {
                row: start_position.row,
                column: start_position.column,
            },
            &TextPosition {
                row: end_position.row,
                column: end_position.column,
            },
        )
    }

    pub fn as_StartEndIndex(&self) -> (usize, usize) {
        if self.length == 0 {
            return (self.index, self.index);
        } else {
            return (self.index, self.index + self.length - 1);
        }
    }

    pub fn as_StartEndTextPosition(&self, text: &String) -> Option<(TextPosition, TextPosition)> {
        let (start_index, end_index) = self.as_StartEndIndex();

        if let (Some(start_position), Some(end_position)) = (
            TextPosition::from_TextIndex(text, start_index),
            TextPosition::from_TextIndex(text, end_index),
        ) {
            return Some((start_position, end_position));
        } else {
            return None;
        }
    }

    pub fn as_StartEndTSPoint(
        &self,
        text: &String,
    ) -> Option<(tree_sitter::Point, tree_sitter::Point)> {
        if let Some((start_position, end_position)) = self.as_StartEndTextPosition(text) {
            Some((start_position.as_TSPoint(), end_position.as_TSPoint()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests_TextRange {
    use crate::core_engine::rules::utils::text_types::{TextPosition, TextRange};

    use pretty_assertions::assert_eq;

    #[test]
    fn test_TextRange_from_StartEndIndex() {
        // Given a string "Hello World!" and a start and end index of 5 and 9,
        // ===============|0--->5-->9-|==================
        // The length is 5 characters, including the start and end index'es characters.
        // Converting from range to indexes, the end index is supposed to be the last character
        // that is included by the length.
        let start_index = 5;
        let end_index = 9;

        let range = TextRange::from_StartEndIndex(start_index, end_index);

        assert_eq!(range.index, 5);
        assert_eq!(range.length, 5);

        assert_eq!(range.as_StartEndIndex(), (start_index, end_index));
    }

    #[test]
    fn test_TextRange_from_StartEndTextPosition_one_line() {
        let text = "Hello, World!";
        //                     |--->| <- Length is 6 characters
        let start_position = TextPosition::new(0, 5);
        let end_position = TextPosition::new(0, 10);
        let range_option =
            TextRange::from_StartEndTextPosition(&text.to_string(), &start_position, &end_position);

        assert_eq!(range_option.is_some(), true);

        let range = range_option.unwrap();

        assert_eq!(range.index, 5);
        assert_eq!(range.length, 6);
    }

    #[test]
    fn test_TextRange_from_StartEndTextPosition_multi_line() {
        let text = "He\nll\no, Wo\nrld!";
        //                    |-- ------ ->| <- Length 12 ('\n' is one character)
        let start_position = TextPosition::new(1, 0);
        let end_position = TextPosition::new(3, 2);
        let range_option =
            TextRange::from_StartEndTextPosition(&text.to_string(), &start_position, &end_position);

        assert_eq!(range_option.is_some(), true);

        let range = range_option.unwrap();

        assert_eq!(range.index, 3);
        assert_eq!(range.length, 12);
    }

    #[test]
    fn test_TextRange_from_StartEndTextPosition_col_too_big() {
        let text = "He\nll\no, Wo\nrld!";
        let start_position = TextPosition::new(1, 100);
        let end_position = TextPosition::new(3, 2);
        let range_option =
            TextRange::from_StartEndTextPosition(&text.to_string(), &start_position, &end_position);

        assert_eq!(range_option.is_some(), false);
    }

    #[test]
    fn test_TextRange_from_StartEndTextPosition_row_too_big() {
        let text = "He\nll\no, Wo\nrld!";
        let start_position = TextPosition::new(1, 1);
        let end_position = TextPosition::new(100, 2);
        let range_option =
            TextRange::from_StartEndTextPosition(&text.to_string(), &start_position, &end_position);

        assert_eq!(range_option.is_some(), false);
    }

    #[test]
    fn test_TextRange_as_StartEndIndex_one_line() {
        // "Hello, World!";
        //  |--->| <- Length is 6 characters, start is 0, end is 5
        let range = TextRange::new(0, 6);

        assert_eq!(range.as_StartEndIndex(), (0, 5));
    }

    #[test]
    fn test_TextRange_as_StartEndIndex_index_and_length_zero() {
        // "Hello, World!";
        // >| <- Length is 0 characters, start is 0, end is 0
        let range = TextRange::new(0, 0);

        assert_eq!(range.as_StartEndIndex(), (0, 0));
    }

    #[test]
    fn test_TextRange_as_StartEndIndex_length_zero() {
        // "Hello, World!";
        // >| <- Length is 0 characters, start is 25, end is 25
        let range = TextRange::new(25, 0);

        assert_eq!(range.as_StartEndIndex(), (25, 25));
    }

    // test TextRange as_StartEndPosition
    #[test]
    fn test_TextRange_as_StartEndTextPosition_one_line() {
        let text = "Hello, World!";
        //                |--->| <- Length is 6 characters, start is [0,0], end is [0,5]
        let range = TextRange::new(0, 6);

        let range_option = range.as_StartEndTextPosition(&text.to_string());

        assert_eq!(range_option.is_some(), true);

        let (start_pos, end_pos) = range_option.unwrap();

        // Start Position
        assert_eq!(start_pos.row, 0);
        assert_eq!(start_pos.column, 0);

        // End Position
        assert_eq!(end_pos.row, 0);
        assert_eq!(end_pos.column, 5);
    }

    #[test]
    fn test_TextRange_as_StartEndTextPosition_multi_line() {
        let text = "He\nll\no, Wo\nrld!";
        //                    |-- ------ ->| <- Length 12 ('\n' is one character)
        let range = TextRange::new(3, 12);

        let range_option = range.as_StartEndTextPosition(&text.to_string());

        assert_eq!(range_option.is_some(), true);

        let (start_pos, end_pos) = range_option.unwrap();

        // Start Position
        assert_eq!(start_pos.row, 1);
        assert_eq!(start_pos.column, 0);

        // End Position
        assert_eq!(end_pos.row, 3);
        assert_eq!(end_pos.column, 2);
    }
}

pub fn StartEndIndex_from_StartEndTextPosition(
    text: &String,
    start_position: &TextPosition,
    end_position: &TextPosition,
) -> Option<(usize, usize)> {
    if let Some(char_range) =
        TextRange::from_StartEndTextPosition(text, start_position, end_position)
    {
        Some(char_range.as_StartEndIndex())
    } else {
        None
    }
}

pub fn StartEndIndex_from_StartEndTSPoint(
    text: &String,
    start_point: &tree_sitter::Point,
    end_point: &tree_sitter::Point,
) -> Option<(usize, usize)> {
    StartEndIndex_from_StartEndTextPosition(
        text,
        &TextPosition::from_TSPoint(start_point),
        &TextPosition::from_TSPoint(end_point),
    )
}

pub fn StartEndTextPosition_from_StartEndIndex(
    text: &String,
    start_index: usize,
    end_index: usize,
) -> Option<(TextPosition, TextPosition)> {
    let range = TextRange::from_StartEndIndex(start_index, end_index);

    range.as_StartEndTextPosition(&text)
}

pub fn StartEndTextPosition_from_StartEndTSPoint(
    start_point: &tree_sitter::Point,
    end_point: &tree_sitter::Point,
) -> (TextPosition, TextPosition) {
    (
        TextPosition::from_TSPoint(start_point),
        TextPosition::from_TSPoint(end_point),
    )
}

pub fn StartEndTSPoint_from_StartEndIndex(
    text: &String,
    start_index: usize,
    end_index: usize,
) -> Option<(tree_sitter::Point, tree_sitter::Point)> {
    if let Some((start_pos, end_pos)) =
        StartEndTextPosition_from_StartEndIndex(text, start_index, end_index)
    {
        Some((start_pos.as_TSPoint(), end_pos.as_TSPoint()))
    } else {
        None
    }
}

pub fn StartEndTSPoint_from_StartEndTextPosition(
    start_position: &TextPosition,
    end_position: &TextPosition,
) -> (tree_sitter::Point, tree_sitter::Point) {
    (
        TextPosition::as_TSPoint(start_position),
        TextPosition::as_TSPoint(end_position),
    )
}

#[cfg(test)]
mod tests_TextConversions {
    use crate::core_engine::rules::utils::text_types::{
        StartEndIndex_from_StartEndTSPoint, StartEndIndex_from_StartEndTextPosition, TextPosition,
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn test_StartEndIndex_from_StartEndTextPosition_one_line() {
        let text = "Hello, World!";
        //                |--->|  start index 0, end index 5
        let start_position = TextPosition::new(0, 0);
        let end_position = TextPosition::new(0, 5);
        let result = StartEndIndex_from_StartEndTextPosition(
            &text.to_string(),
            &start_position,
            &end_position,
        );

        assert_eq!(result.is_some(), true);

        let (start_index, end_index) = result.unwrap();

        assert_eq!(start_index, 0);
        assert_eq!(end_index, 5);
    }

    #[test]
    fn test_StartEndIndex_from_StartEndTextPosition_two_lines() {
        let text = "Hello, World!\nHello, World!";
        //                |------------- ---->|  start index 0, end index 19
        let start_position = TextPosition::new(0, 0);
        let end_position = TextPosition::new(1, 5);
        let result = StartEndIndex_from_StartEndTextPosition(
            &text.to_string(),
            &start_position,
            &end_position,
        );

        assert_eq!(result.is_some(), true);

        let (start_index, end_index) = result.unwrap();

        assert_eq!(start_index, 0);
        assert_eq!(end_index, 19);
    }

    // Write test for StartEndIndex_from_StartEndTSPoint
    #[test]
    fn test_StartEndIndex_from_StartEndTSPoint_one_line() {
        let text = "Hello, World!";
        let start_point = tree_sitter::Point::new(0, 0);
        let end_point = tree_sitter::Point::new(0, 5);
        let result =
            StartEndIndex_from_StartEndTSPoint(&text.to_string(), &start_point, &end_point);

        assert_eq!(result.is_some(), true);

        let (start_index, end_index) = result.unwrap();

        assert_eq!(start_index, 0);
        assert_eq!(end_index, 5);
    }
}

pub fn get_index_of_next_row(index: usize, text: &String) -> Option<(usize)> {
    let mut i = 0;
    for c in text.chars().skip(index) {
        if c == '\n' {
            return Some(index + i + 1);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests_Text {
    use crate::core_engine::rules::get_index_of_next_row;

    #[test]
    fn test_index_of_next_row() {
        // No new row in text
        assert_eq!(get_index_of_next_row(5, &"Hello, World!".to_string()), None);

        // return index at new row and keep end index
        let text = "Hello,
      World!"
            .to_string();
        assert_eq!(get_index_of_next_row(5, &text), Some(7));

        // return index at new row and keep end index
        assert_eq!(
            get_index_of_next_row(5, &"Hello test,\n Wor!ld!\n".to_string()),
            Some(12)
        );
    }
}
