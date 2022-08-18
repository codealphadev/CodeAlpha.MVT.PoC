use crate::core_engine::rules::TextPosition;

pub struct Utf16LinesIterator {
    curr: Vec<u16>,
    next: Vec<u16>,
}
impl Iterator for Utf16LinesIterator {
    type Item = Vec<u16>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_empty() {
            return None;
        }

        let mut last_char_was_carriage_return = false;
        for (i, c) in self.curr.iter().enumerate() {
            if *c == '\r' as u16 {
                last_char_was_carriage_return = true;
            }
            if *c == '\n' as u16 {
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
pub fn utf16_lines(text: &Vec<u16>) -> Utf16LinesIterator {
    Utf16LinesIterator {
        curr: text.to_vec(),
        next: vec![],
    }
}

pub fn utf16_is_whitespace(c: &u16) -> bool {
    if let Ok(u8_c) = u8::try_from(*c) {
        if (u8_c as char).is_whitespace() {
            return true;
        }
    }
    false
}

// Used in tests
#[allow(dead_code)]
pub fn utf16_bytes_count(c: &Vec<u16>) -> usize {
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
    use super::utf16_lines;

    #[test]
    fn no_new_line() {
        let text: Vec<u16> = "Hello ✌🏻".encode_utf16().collect();
        let expected: Vec<Vec<u16>> = vec![text.clone()];
        let lines: Vec<Vec<u16>> = utf16_lines(&text).collect();
        println!("{:?}", lines);
        assert_eq!(lines, expected);
    }

    #[test]
    fn one_new_line() {
        let text: Vec<u16> = "Hello ✌🏻\nWorld".encode_utf16().collect();
        let expected: Vec<Vec<u16>> = vec![
            "Hello ✌🏻".encode_utf16().collect(),
            "World".encode_utf16().collect(),
        ];
        let lines: Vec<Vec<u16>> = utf16_lines(&text).collect();
        assert_eq!(lines, expected);
    }

    #[test]
    fn carriage_return() {
        let text: Vec<u16> = "Hello ✌🏻\nWorld\r\ntest".encode_utf16().collect();
        let expected: Vec<Vec<u16>> = vec![
            "Hello ✌🏻".encode_utf16().collect(),
            "World".encode_utf16().collect(),
            "test".encode_utf16().collect(),
        ];
        let lines: Vec<Vec<u16>> = utf16_lines(&text).collect();
        println!("{:?}", lines);
        assert_eq!(lines, expected);
    }

    #[test]
    fn empty_vec() {
        let text: Vec<u16> = vec![];
        let expected: Vec<Vec<u16>> = vec![];
        let lines: Vec<Vec<u16>> = utf16_lines(&text).collect();
        assert_eq!(lines, expected);
    }
}
