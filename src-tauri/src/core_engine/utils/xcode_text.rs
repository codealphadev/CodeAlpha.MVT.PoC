use crate::core_engine::rules::TextPosition;

pub type XcodeChar = u16;
pub type XcodeText = Vec<XcodeChar>;
pub type XcodeTextRows = Vec<XcodeText>;

#[derive(PartialEq, Clone, Debug)]
pub struct XcodeTextLinesIterator {
    curr: XcodeText,
    next: Option<XcodeText>,
}
impl Iterator for XcodeTextLinesIterator {
    type Item = XcodeText;
    fn next(&mut self) -> Option<Self::Item> {
        let next = if let Some(next) = &self.next {
            next
        } else {
            self.next = None;
            return None;
        };

        if next.is_empty() {
            self.next = None;
            return Some(vec![]);
        }

        let mut index_last_carriage_return = None;
        for (i, c) in next.iter().enumerate() {
            if *c == '\r' as XcodeChar {
                index_last_carriage_return = Some(i);
            }
            if *c == '\n' as XcodeChar {
                let mut chars_to_remove = 0;
                if let Some(last_char_was_carriage_return) = index_last_carriage_return {
                    if last_char_was_carriage_return == i - 1 {
                        chars_to_remove = 1;
                    }
                }

                let curr = next[..i - chars_to_remove].to_vec();
                let rest = next[i + 1..].to_vec();

                self.curr = curr.clone();
                self.next = Some(rest);
                return Some(curr);
            }
        }

        // No newline found
        let response = next.clone();
        self.curr = vec![];
        self.next = None;
        Some(response)
    }
}

pub fn xcode_text_rows(text: &XcodeText) -> XcodeTextLinesIterator {
    XcodeTextLinesIterator {
        curr: vec![],
        next: Some(text.to_vec()),
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
    fn newline_last_char() {
        let text: XcodeText = "o\n".encode_utf16().collect();
        let expected: XcodeTextRows = vec!["o".encode_utf16().collect(), vec![]];
        let lines: XcodeTextRows = xcode_text_rows(&text).collect();
        assert_eq!(lines, expected);
    }

    #[test]
    fn only_newline() {
        let text: XcodeText = "\n".encode_utf16().collect();
        let expected: XcodeTextRows = vec![vec![], vec![]];
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
    fn carriage_return_wrong_place() {
        let text: XcodeText = "He\rllo\nWorld".encode_utf16().collect();
        let expected: XcodeTextRows = vec![
            "He\rllo".encode_utf16().collect(),
            "World".encode_utf16().collect(),
        ];
        let lines: XcodeTextRows = xcode_text_rows(&text).collect();
        println!("{:?}", lines);
        assert_eq!(lines, expected);
    }

    #[test]
    fn empty_vec() {
        let text: XcodeText = vec![];
        let expected: XcodeTextRows = vec![vec![]];
        let lines: XcodeTextRows = xcode_text_rows(&text).collect();
        assert_eq!(lines, expected);
    }
}
