use crate::core_engine::rules::TextPosition;

pub type XcodeChar = u16;
pub type XcodeText = Vec<XcodeChar>;
pub type XcodeTextRows = Vec<XcodeText>;

pub struct XcodeTextLinesIterator {
    curr: XcodeText,
    next: XcodeText,
}
impl Iterator for XcodeTextLinesIterator {
    type Item = XcodeText;
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_empty() {
            return None;
        }

        let mut last_char_was_carriage_return = false;
        for (i, c) in self.curr.iter().enumerate() {
            if *c == '\r' as XcodeChar {
                last_char_was_carriage_return = true;
            }
            if *c == '\n' as XcodeChar {
                let mut rest = self.curr.split_off(if last_char_was_carriage_return {
                    i - 1
                } else {
                    i
                });
                let response = self.curr.clone();
                let chars_to_remove = if last_char_was_carriage_return { 2 } else { 1 };
                self.curr = rest.split_off(chars_to_remove); // remove the newline
                self.next = vec![];
                return Some(response);
            }
        }

        let response = self.curr.clone();
        self.curr = vec![];
        self.next = vec![];
        Some(response)
    }
}

pub fn xcode_text_rows(text: &XcodeText) -> XcodeTextLinesIterator {
    XcodeTextLinesIterator {
        curr: text.to_vec(),
        next: vec![],
    }
}

pub fn xcode_char_is_whitespace(c: &XcodeChar) -> bool {
    if let Ok(u8_c) = u8::try_from(*c) {
        if (u8_c as char).is_whitespace() {
            return true;
        }
    }
    false
}

// Used in tests
#[allow(dead_code)]
pub fn utf16_bytes_count(c: &XcodeText) -> usize {
    c.len() * 2
}

pub fn utf16_treesitter_point_to_position(point: &tree_sitter::Point) -> TextPosition {
    TextPosition {
        row: point.row,
        column: point.column / 2,
    }
}

pub fn utf16_position_to_tresitter_point(point: &TextPosition) -> tree_sitter::Point {
    tree_sitter::Point {
        row: point.row,
        column: point.column * 2,
    }
}

#[cfg(test)]
mod tests_utf16_lines {
    use super::*;

    #[test]
    fn no_new_line() {
        let text: XcodeText = "Hello ✌🏻".encode_utf16().collect();
        let expected: XcodeTextRows = vec![text.clone()];
        let lines: XcodeTextRows = xcode_text_rows(&text).collect();
        println!("{:?}", lines);
        assert_eq!(lines, expected);
    }

    #[test]
    fn one_new_line() {
        let text: XcodeText = "Hello ✌🏻\nWorld".encode_utf16().collect();
        let expected: XcodeTextRows = vec![
            "Hello ✌🏻".encode_utf16().collect(),
            "World".encode_utf16().collect(),
        ];
        let lines: XcodeTextRows = xcode_text_rows(&text).collect();
        assert_eq!(lines, expected);
    }

    #[test]
    fn carriage_return() {
        let text: XcodeText = "Hello ✌🏻\nWorld\r\ntest".encode_utf16().collect();
        let expected: XcodeTextRows = vec![
            "Hello ✌🏻".encode_utf16().collect(),
            "World".encode_utf16().collect(),
            "test".encode_utf16().collect(),
        ];
        let lines: XcodeTextRows = xcode_text_rows(&text).collect();
        println!("{:?}", lines);
        assert_eq!(lines, expected);
    }

    #[test]
    fn empty_vec() {
        let text: XcodeText = vec![];
        let expected: XcodeTextRows = vec![];
        let lines: XcodeTextRows = xcode_text_rows(&text).collect();
        assert_eq!(lines, expected);
    }
}
