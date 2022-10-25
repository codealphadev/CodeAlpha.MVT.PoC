#![allow(dead_code)]

use std::time::Instant;

use crate::core_engine::utils::{TextPosition, TextRange, XcodeText};
use similar::{capture_diff_slices_deadline, Algorithm, DiffOp};
use tree_sitter::InputEdit;
#[derive(Clone, Debug)]
pub struct TextDiff2 {
    pub added_char_count: usize,
    pub removed_char_count: usize,
    pub start_index: usize,
}

pub fn detect_input_edits(
    old_string: &XcodeText,
    new_string: &XcodeText,
    deadline_ms: u64,
) -> Vec<InputEdit> {
    let mut edits: Vec<TextDiff2> = Vec::new();

    // https://docs.rs/similar/latest/similar/#deadlines-and-performance
    // "Due to the recursive, divide and conquer nature of Myer’s diff you will still get a pretty decent diff in many cases if the deadline is reached"
    let deadline = Instant::now() + std::time::Duration::from_millis(deadline_ms);

    let ops = capture_diff_slices_deadline(
        Algorithm::Myers,
        &old_string.text,
        &new_string.text,
        Some(deadline),
    );
    dbg!(ops.len());

    for diff in ops {
        match diff {
            DiffOp::Equal {
                old_index: _,
                new_index: _,
                len: _,
            } => {}
            DiffOp::Insert {
                old_index,
                new_index: _,
                new_len,
            } => {
                edits.push(TextDiff2 {
                    added_char_count: new_len,
                    removed_char_count: 0,
                    start_index: old_index,
                });
            }
            DiffOp::Delete {
                old_index,
                old_len,
                new_index: _,
            } => {
                edits.push(TextDiff2 {
                    added_char_count: 0,
                    removed_char_count: old_len,
                    start_index: old_index,
                });
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index: _,
                new_len,
            } => {
                edits.push(TextDiff2 {
                    added_char_count: new_len,
                    removed_char_count: old_len,
                    start_index: old_index,
                });
            }
        }
    }

    construct_InputEdits_from_detected_edits(old_string, new_string, &edits)
}

fn construct_InputEdits_from_detected_edits(
    old_string: &XcodeText,
    new_string: &XcodeText,
    detected_edits: &Vec<TextDiff2>,
) -> Vec<InputEdit> {
    let mut input_edits: Vec<InputEdit> = Vec::new();
    for edit in detected_edits.iter() {
        let edit_range_before = TextRange::new(edit.start_index, edit.removed_char_count);
        let edit_range_after = TextRange::new(edit.start_index, edit.added_char_count);

        if let (Some(old_pts), Some(new_pts), Some(start_position)) = (
            edit_range_before.as_StartEndTSPoint(&old_string),
            edit_range_after.as_StartEndTSPoint(&new_string),
            TextPosition::from_TextIndex(old_string, edit.start_index),
        ) {
            input_edits.push(InputEdit {
                start_byte: edit.start_index * 2, // UTF-16
                old_end_byte: (edit.start_index + edit.removed_char_count) * 2,
                new_end_byte: (edit.start_index + edit.added_char_count) * 2,
                start_position: start_position.as_TSPoint(),
                old_end_position: old_pts.1,
                new_end_position: new_pts.1,
            });
        }
    }
    input_edits
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    fn test_fn(
        old_string: &str,
        new_string: &str,
        start_index: usize,
        old_end_index: usize,
        new_end_index: usize,
        start_position: TextPosition,
        old_end_position: TextPosition,
        new_end_position: TextPosition,
    ) {
        let input_edits = detect_input_edits(
            &XcodeText::from_str(&old_string),
            &XcodeText::from_str(&new_string),
            1000,
        );
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, start_index * 2);
        assert_eq!(input_edits[0].old_end_byte, old_end_index * 2);
        assert_eq!(input_edits[0].new_end_byte, new_end_index * 2);
        assert_eq!(
            TextPosition::from_TSPoint(&input_edits[0].start_position),
            start_position
        );
        assert_eq!(
            TextPosition::from_TSPoint(&input_edits[0].old_end_position),
            old_end_position
        );
        assert_eq!(
            TextPosition::from_TSPoint(&input_edits[0].new_end_position),
            new_end_position
        );
    }

    #[test]
    fn remove_at_end_of_line() {
        test_fn(
            "abcdef",
            "abcd",
            4,
            6,
            4,
            TextPosition { row: 0, column: 4 },
            TextPosition { row: 0, column: 6 },
            TextPosition { row: 0, column: 4 },
        );
    }

    #[test]
    fn remove_with_emoji() {
        test_fn(
            "abcdef😊a",
            "abcd",
            4,
            9,
            4,
            TextPosition { row: 0, column: 4 },
            TextPosition { row: 0, column: 9 },
            TextPosition { row: 0, column: 4 },
        );
    }

    #[test]
    fn remove_with_emoji_end() {
        test_fn(
            "abcdef😊",
            "abcd",
            4,
            8,
            4,
            TextPosition { row: 0, column: 4 },
            TextPosition { row: 0, column: 8 },
            TextPosition { row: 0, column: 4 },
        );
    }

    #[test]
    fn add_with_emoji() {
        test_fn(
            "abcdef😊a",
            "abcdef😊aBC",
            9,
            9,
            11,
            TextPosition { row: 0, column: 9 },
            TextPosition { row: 0, column: 9 },
            TextPosition { row: 0, column: 11 },
        );
    }

    #[test]
    fn replace_with_emoji() {
        test_fn(
            "abc😊aBCd",
            "abc😊aXXYYYd",
            6,
            8,
            11,
            TextPosition { row: 0, column: 6 },
            TextPosition { row: 0, column: 8 },
            TextPosition { row: 0, column: 11 },
        );
    }

    #[test]
    fn replace_beginning_of_file() {
        test_fn(
            "let x = 1; cansole.lug(x);",
            "const x = 1; cansole.lug(x);",
            0,
            2,
            4,
            TextPosition { row: 0, column: 0 },
            TextPosition { row: 0, column: 2 },
            TextPosition { row: 0, column: 4 },
        );
    }

    #[test]
    fn replace_in_middle_of_line() {
        test_fn(
            "fn test(XXX) {}",
            "fn test(a: u32) {}",
            8,
            11,
            14,
            TextPosition { row: 0, column: 8 },
            TextPosition { row: 0, column: 11 },
            TextPosition { row: 0, column: 14 },
        );
    }

    #[test]
    fn replace_whole_line() {
        test_fn(
            "fn test0(a: u32) {}\n//row\nfn test2(a: u32) {}",
            "fn test0(a: u32) {}\nfn test1(a: u32) {}\nfn test2(a: u32) {}",
            20,
            25,
            39,
            TextPosition { row: 1, column: 0 },
            TextPosition { row: 1, column: 5 },
            TextPosition { row: 1, column: 19 },
        );
    }
}
