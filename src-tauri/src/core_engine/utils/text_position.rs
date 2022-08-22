use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::utils::XcodeText;

/// A position in a multi-line text document, in terms of rows and columns.
/// Rows and columns are zero-based.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
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
        XcodeText::treesitter_point_to_position(tree_sitter_point)
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
    pub fn from_TextIndex(text: &XcodeText, index: usize) -> Option<TextPosition> {
        if index > text.len() {
            return None;
        }

        let mut i = 0;
        for (row_i, row) in text.rows_iter().enumerate() {
            for col_i in 0..=row.len() {
                if i == index {
                    return Some(TextPosition {
                        row: row_i,
                        column: col_i,
                    });
                }
                i += 1;
            }
        }
        None
    }

    pub fn as_TSPoint(&self) -> tree_sitter::Point {
        XcodeText::position_to_tresitter_point(self)
    }

    pub fn as_TextIndex(&self, text: &XcodeText) -> Option<usize> {
        self.as_TextIndex_stay_on_line(text, false)
    }

    pub fn as_TextIndex_stay_on_line(&self, text: &XcodeText, stay_on_line: bool) -> Option<usize> {
        let mut i = 0;
        for (row_i, row) in text.rows_iter().enumerate() {
            for col_i in 0..=row.len() {
                if self.row == row_i && self.column == col_i {
                    return Some(i);
                }
                i += 1;
            }
            if stay_on_line && self.row == row_i {
                return Some(i - 1);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::core_engine::utils::{TextPosition, XcodeText};

    #[test]
    fn TextPosition_from_TextIndex_respects_new_line_character() {
        let text = XcodeText::from_str("\nHello, \nWorld!");
        let index = 12; // ... starting from zero, so the 13th character, which is the 'l'
        assert_eq!(text[index], 'l' as u16);

        let position_option = TextPosition::from_TextIndex(&text, index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 2);
        assert_eq!(position.column, 3);
    }

    #[test]
    fn TextPosition_from_TextIndex_one_line() {
        let text = XcodeText::from_str("Hello, World!");
        let index = 5;
        let position_option = TextPosition::from_TextIndex(&text, index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 0);
        assert_eq!(position.column, 5);
    }

    #[test]
    fn TextPosition_from_TextIndex_two_lines() {
        let text = XcodeText::from_str("Hello, World!\nGoodbye, World!");
        let index = 20; // ... starting from zero, so the 21th character, which is the 'e'
        assert_eq!(text[index], 'e' as u16);
        let position_option = TextPosition::from_TextIndex(&text, index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 1);
        assert_eq!(position.column, 6);
    }

    #[test]
    fn TextPosition_from_TextIndex_none_with_emojis() {
        let text = XcodeText::from_str("Hell😊, W😊rld!");
        //       is 4 bytes ->|   |<- is 4 bytes
        let index = 5;
        let position_option = TextPosition::from_TextIndex(&text, index);

        assert_eq!(position_option.is_some(), true);

        let position = position_option.unwrap();

        assert_eq!(position.row, 0);
        assert_eq!(position.column, 5);
    }

    #[test]
    fn TextPosition_from_TextIndex_last_position() {
        let text = XcodeText::from_str("Hello, World!");
        let index = 13;
        let position = TextPosition::from_TextIndex(&text, index).unwrap();

        assert_eq!(position.row, 0);
        assert_eq!(position.column, 13);
    }

    #[test]
    fn TextPosition_from_TextIndex_too_far() {
        let text = XcodeText::from_str("Hello, World!");
        let index = 14;
        let position_option = TextPosition::from_TextIndex(&text, index);

        assert_eq!(position_option.is_none(), true);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex() {
        let text = XcodeText::from_str("Hello, World!");
        let position = TextPosition::new(0, 5);
        let index_option = position.as_TextIndex(&text);

        assert_eq!(index_option.is_some(), true);

        let index = index_option.unwrap();

        assert_eq!(index, 5);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_with_emojis() {
        let text = XcodeText::from_str("Hell😊, W😊rld!");
        //       is 4 bytes ->|   |<- is 4 bytes
        let position = TextPosition::new(0, 5);
        let index_option = position.as_TextIndex(&text);

        assert_eq!(index_option.is_some(), true);

        let index = index_option.unwrap();

        assert_eq!(index, 5);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_multi_line() {
        let text = XcodeText::from_str("Hello,\n World\n!");
        let position = TextPosition::new(2, 0);
        let index_option = position.as_TextIndex(&text);

        assert_eq!(index_option.is_some(), true);

        let index = index_option.unwrap();

        assert_eq!(index, 14);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far() {
        let text = XcodeText::from_str("Hello, World!");
        let position = TextPosition::new(0, 14);
        let index_option = position.as_TextIndex(&text);

        assert_eq!(index_option.is_none(), true);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far_multiline_stay_on_line() {
        let text = XcodeText::from_str("Hello,\nWorld!\n");
        let position = TextPosition::new(1, 7);
        let index_option = position.as_TextIndex_stay_on_line(&text, true);

        assert_eq!(index_option.unwrap(), 13);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far_multiline_stay_on_line_last() {
        let text = XcodeText::from_str("Hello,\nWorld!\n");
        let position = TextPosition::new(2, 7);
        let index_option = position.as_TextIndex_stay_on_line(&text, true);

        assert_eq!(index_option.unwrap(), 14);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_stay_on_line_empty_string() {
        let text = XcodeText::from_str("");
        let position = TextPosition::new(0, 7);
        let index_option = position.as_TextIndex_stay_on_line(&text, true);

        assert_eq!(index_option.unwrap(), 0);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_stay_on_line_out_of_range() {
        let text = XcodeText::from_str("Hello");
        let position = TextPosition::new(1, 7);
        let index_option = position.as_TextIndex_stay_on_line(&text, true);

        assert!(index_option.is_none());
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far_multiline_stay_on_line_with_emojis() {
        let text = XcodeText::from_str("Hell😊,\nW😊rld!\n");
        let position = TextPosition::new(0, 100);
        let index_option = position.as_TextIndex_stay_on_line(&text, true);

        assert_eq!(index_option.unwrap(), 7);
    }

    #[test]
    fn convert_TextPosition_as_TextIndex_too_far_multiline_stay_on_line_false() {
        let text = XcodeText::from_str("Hello,\nWorld!\n");
        let position = TextPosition::new(0, 100);
        let index_option = position.as_TextIndex_stay_on_line(&text, false);

        assert!(index_option.is_none());
    }
}
