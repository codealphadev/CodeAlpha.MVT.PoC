use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::utils::{TextPosition, XcodeText};

/// A range in a multi-line text document, in terms of index and length.
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
            length: end_index - start_index,
        }
    }

    pub fn from_StartEndTextPosition(
        text: &XcodeText,
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
        text: &XcodeText,
        start_position: &tree_sitter::Point,
        end_position: &tree_sitter::Point,
    ) -> Option<TextRange> {
        Self::from_StartEndTextPosition(
            text,
            &XcodeText::treesitter_point_to_position(start_position),
            &XcodeText::treesitter_point_to_position(end_position),
        )
    }

    pub fn as_StartEndIndex(&self) -> (usize, usize) {
        if self.length == 0 {
            return (self.index, self.index);
        } else {
            return (self.index, self.index + self.length);
        }
    }

    pub fn as_StartEndTextPosition(
        &self,
        text: &XcodeText,
    ) -> Option<(TextPosition, TextPosition)> {
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
        text: &XcodeText,
    ) -> Option<(tree_sitter::Point, tree_sitter::Point)> {
        if let Some((start_position, end_position)) = self.as_StartEndTextPosition(text) {
            Some((start_position.as_TSPoint(), end_position.as_TSPoint()))
        } else {
            None
        }
    }

    pub fn includes_index(&self, index: usize) -> bool {
        if self.length == 0 {
            return index == self.index;
        } else {
            return index >= self.index && index < self.index + self.length;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core_engine::utils::{TextPosition, TextRange, XcodeText};

    use pretty_assertions::assert_eq;

    #[test]
    fn TextRange_from_StartEndIndex() {
        // Given a string "Hello World!" and a start and end index of 5 and 9,
        // ===============|0--->5-->9-|==================
        // The length is 5 characters, including the start and end index'es characters.
        // Converting from range to indexes, the end index is supposed to be the last character
        // that is included by the length.
        let start_index = 5;
        let end_index = 9;

        let range = TextRange::from_StartEndIndex(start_index, end_index);

        assert_eq!(range.index, 5);
        assert_eq!(range.length, 4);

        assert_eq!(range.as_StartEndIndex(), (start_index, end_index));
    }

    #[test]
    fn TextRange_from_StartEndTextPosition_one_line() {
        let text = XcodeText::from_str("Hello, World!");
        //                     |--->| <- Length is 6 characters
        let start_position = TextPosition::new(0, 5);
        let end_position = TextPosition::new(0, 10);
        let range_option =
            TextRange::from_StartEndTextPosition(&text, &start_position, &end_position);

        assert_eq!(range_option.is_some(), true);

        let range = range_option.unwrap();

        assert_eq!(range.index, 5);
        assert_eq!(range.length, 5);
    }

    #[test]
    fn TextRange_from_StartEndTextPosition_multi_line() {
        let text = XcodeText::from_str("He\nll\no, Wo\nrld!");
        //                    |-- ------ ->| <- Length 12 ('\n' is one character)
        let start_position = TextPosition::new(1, 0);
        let end_position = TextPosition::new(3, 2);
        let range_option =
            TextRange::from_StartEndTextPosition(&text, &start_position, &end_position);

        assert_eq!(range_option.is_some(), true);

        let range = range_option.unwrap();

        assert_eq!(range.index, 3);
        assert_eq!(range.length, 11);
    }

    #[test]
    fn TextRange_from_StartEndTextPosition_col_too_big() {
        let text = XcodeText::from_str("He\nll\no, Wo\nrld!");
        let start_position = TextPosition::new(1, 100);
        let end_position = TextPosition::new(3, 2);
        let range_option =
            TextRange::from_StartEndTextPosition(&text, &start_position, &end_position);

        assert_eq!(range_option.is_some(), false);
    }

    #[test]
    fn TextRange_from_StartEndTextPosition_row_too_big() {
        let text = XcodeText::from_str("He\nll\no, Wo\nrld!");
        let start_position = TextPosition::new(1, 1);
        let end_position = TextPosition::new(100, 2);
        let range_option =
            TextRange::from_StartEndTextPosition(&text, &start_position, &end_position);

        assert_eq!(range_option.is_some(), false);
    }

    #[test]
    fn TextRange_as_StartEndIndex_one_line() {
        // "Hello, World!";
        //  |---->| <- Length is 6 characters, start is 0, end is 6
        let range = TextRange::new(0, 6);

        assert_eq!(range.as_StartEndIndex(), (0, 6));
    }

    #[test]
    fn TextRange_as_StartEndIndex_index_and_length_zero() {
        // "Hello, World!";
        // >| <- Length is 0 characters, start is 0, end is 0
        let range = TextRange::new(0, 0);

        assert_eq!(range.as_StartEndIndex(), (0, 0));
    }

    #[test]
    fn TextRange_as_StartEndIndex_length_zero() {
        // "Hello, World!";
        // >| <- Length is 0 characters, start is 25, end is 25
        let range = TextRange::new(25, 0);

        assert_eq!(range.as_StartEndIndex(), (25, 25));
    }

    // test TextRange as_StartEndPosition
    #[test]
    fn TextRange_as_StartEndTextPosition_one_line() {
        let text = XcodeText::from_str("Hello, World!");
        //                |---->| <- Length is 6 characters, start is [0,0], end is [0,6]
        let range = TextRange::new(0, 6);

        let range_option = range.as_StartEndTextPosition(&text);

        assert_eq!(range_option.is_some(), true);

        let (start_pos, end_pos) = range_option.unwrap();

        // Start Position
        assert_eq!(start_pos.row, 0);
        assert_eq!(start_pos.column, 0);

        // End Position
        assert_eq!(end_pos.row, 0);
        assert_eq!(end_pos.column, 6);
    }

    #[test]
    fn TextRange_as_StartEndTextPosition_multi_line() {
        let text = XcodeText::from_str("He\nll\no, Wo\nrld!");
        //                    |-- ------ -->| <- Length 12 ('\n' is one character)
        let range = TextRange::new(3, 12);

        let range_option = range.as_StartEndTextPosition(&text);

        assert_eq!(range_option.is_some(), true);

        let (start_pos, end_pos) = range_option.unwrap();

        // Start Position
        assert_eq!(start_pos.row, 1);
        assert_eq!(start_pos.column, 0);

        // End Position
        assert_eq!(end_pos.row, 3);
        assert_eq!(end_pos.column, 3);
    }

    #[test]
    fn TextRange_includes_index() {
        let range = TextRange::new(3, 5);
        assert!(range.includes_index(3));
        assert!(range.includes_index(7));
    }

    #[test]
    fn TextRange_includes_index_false() {
        let range = TextRange::new(3, 5);
        assert!(!range.includes_index(2));
        assert!(!range.includes_index(8));
    }
}
