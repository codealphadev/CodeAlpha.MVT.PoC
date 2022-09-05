use serde::{Deserialize, Serialize};
use ts_rs::TS;

use std::{
    ops::{Deref, DerefMut},
    slice,
};

use super::TextPosition;

pub type XcodeChar = u16;
pub type XcodeTextRows = Vec<Vec<XcodeChar>>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
pub struct XcodeText {
    pub text: Vec<XcodeChar>,
    pub rows: Vec<Vec<XcodeChar>>,
}

impl<'a> XcodeText {
    pub fn new_empty() -> Self {
        Self {
            text: vec![],
            rows: vec![],
        }
    }

    pub fn from_vec(vec: &Vec<XcodeChar>) -> Self {
        Self {
            text: vec.to_vec(),
            rows: XcodeText::create_rows(vec),
        }
    }

    pub fn from_str(str: &str) -> Self {
        Self::from_vec(&str.encode_utf16().collect())
    }

    pub fn from_array(array: &[XcodeChar]) -> Self {
        Self::from_vec(&array.to_vec())
    }

    pub fn as_string(&self) -> String {
        String::from_utf16_lossy(&self.text)
    }

    pub fn utf16_bytes_count(&self) -> usize {
        self.text.len() * 2
    }

    pub fn rows_iter(&self) -> slice::Iter<'_, Vec<u16>> {
        self.rows.iter()
    }

    pub fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            rows: self.rows.clone(),
        }
    }

    pub fn create_rows(text: &Vec<u16>) -> XcodeTextRows {
        if text.is_empty() {
            return vec![vec![]];
        }

        let mut rows = vec![];
        let mut index_last_carriage_return = None;
        let mut last_row_index = 0;
        let mut i = 0;
        while i < text.len() {
            let ch = text[i];
            if ch == '\r' as XcodeChar {
                index_last_carriage_return = Some(i);
            }

            if ch == '\n' as XcodeChar {
                let mut chars_to_remove = 0;
                if let Some(last_char_was_carriage_return) = index_last_carriage_return {
                    if last_char_was_carriage_return == i - 1 {
                        chars_to_remove = 1;
                    }
                }

                rows.push(text[last_row_index..i - chars_to_remove].to_vec());
                last_row_index = i + 1;
                i += chars_to_remove;
            }
            i += 1;
        }
        // last row where no \n exists
        rows.push(text[last_row_index..].to_vec());

        rows
    }

    pub fn char_is_whitespace(c: &XcodeChar) -> bool {
        if let Ok(u8_c) = u8::try_from(*c) {
            if (u8_c as char).is_whitespace() {
                return true;
            }
        }
        false
    }

    pub fn treesitter_point_to_position(point: &tree_sitter::Point) -> TextPosition {
        TextPosition {
            row: point.row,
            column: point.column / 2,
        }
    }

    pub fn position_to_tresitter_point(point: &TextPosition) -> tree_sitter::Point {
        tree_sitter::Point {
            row: point.row,
            column: point.column * 2,
        }
    }
}

impl<'a> IntoIterator for XcodeText {
    type Item = XcodeChar;
    type IntoIter = <Vec<u16> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.text.into_iter()
    }
}

impl<'a> IntoIterator for &'a XcodeText {
    type Item = &'a XcodeChar;
    type IntoIter = slice::Iter<'a, XcodeChar>;

    fn into_iter(self) -> Self::IntoIter {
        self.text.iter()
    }
}

impl<'a> IntoIterator for &'a mut XcodeText {
    type Item = &'a mut XcodeChar;
    type IntoIter = slice::IterMut<'a, XcodeChar>;

    fn into_iter(self) -> Self::IntoIter {
        self.text.iter_mut()
    }
}

impl FromIterator<u16> for XcodeText {
    fn from_iter<I: IntoIterator<Item = u16>>(iter: I) -> Self {
        let mut text = vec![];
        for i in iter {
            text.push(i);
        }
        XcodeText::from_array(&text)
    }
}

impl<'a> AsRef<[u16]> for XcodeText {
    fn as_ref(&self) -> &[u16] {
        &self.text
    }
}

impl<'a> Deref for XcodeText {
    type Target = Vec<XcodeChar>;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl<'a> DerefMut for XcodeText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text
    }
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod rows {
        use super::super::*;

        fn test_fn(text: &str, expected_str: Vec<&str>) {
            let mut expected = vec![];
            for i in 0..expected_str.len() {
                expected.push(expected_str[i].encode_utf16().collect::<Vec<u16>>());
            }
            let rows = XcodeText::create_rows(&text.encode_utf16().collect::<XcodeText>());
            assert_eq!(rows, expected);
        }

        #[test]
        fn no_new_row() {
            test_fn("Hello ✌🏻", vec!["Hello ✌🏻"]);
        }

        #[test]
        fn one_new_row() {
            test_fn("Hello ✌🏻\nWorld", vec!["Hello ✌🏻", "World"]);
        }

        #[test]
        fn newline_last_char() {
            test_fn("o\n", vec!["o", ""]);
        }

        #[test]
        fn only_newline() {
            test_fn("\n", vec!["", ""]);
        }

        #[test]
        fn start_two_empty_lines() {
            test_fn("\n\n          test", vec!["", "", "          test"]);
        }

        #[test]
        fn carriage_return() {
            test_fn(
                "Hello ✌🏻\nWorld\r\ntest",
                vec!["Hello ✌🏻", "World", "test"],
            );
        }

        #[test]
        fn carriage_return_wrong_place() {
            test_fn("He\rllo\nWorld", vec!["He\rllo", "World"]);
        }

        #[test]
        fn empty_vec() {
            test_fn("", vec![""]);
        }
    }
}
