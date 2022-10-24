#![allow(dead_code)]

use crate::core_engine::utils::{TextPosition, TextRange, XcodeText};
use diff;
use tree_sitter::InputEdit;

#[derive(PartialEq, Clone, Debug)]
pub enum DiffType {
    Add,
    Delete,
    Replace,
}

#[derive(Clone, Debug)]
pub struct TextDiff {
    pub diff_type: DiffType,
    pub added_char_sequence: Vec<u16>,
    pub removed_char_sequence: Vec<u16>,
    pub start_index: usize,
}

pub fn detect_input_edits(old_string: &XcodeText, new_string: &XcodeText) -> Vec<InputEdit> {
    let mut edits: Vec<TextDiff> = Vec::new();
    let mut walk_index = 0;

    let mut current_edit: Option<TextDiff> = None;

    for diff in diff::slice(&old_string.text, &new_string.text) {
        match diff {
            diff::Result::Left(l) => {
                if let Some(current_edit) = &mut current_edit {
                    match current_edit.diff_type {
                        DiffType::Add | DiffType::Replace => {
                            // Because both, 'Add' and 'Replace', only get characters added and not removed.
                            // The reason why 'Replace' only gets characters added is a) we always first remove and then add chars
                            // and b) a 'Replace' is only detected if after the removal of consecutive chars there is a add.
                            edits.push(current_edit.clone());
                            current_edit.clone_from(&TextDiff {
                                diff_type: DiffType::Delete,
                                added_char_sequence: vec![],
                                removed_char_sequence: vec![*l],
                                start_index: walk_index,
                            });
                        }
                        DiffType::Delete => {
                            current_edit.removed_char_sequence.push(*l);
                        }
                    }
                } else {
                    current_edit = Some(TextDiff {
                        diff_type: DiffType::Delete,
                        added_char_sequence: Vec::new(),
                        removed_char_sequence: vec![*l],
                        start_index: walk_index,
                    });
                }
            }
            diff::Result::Right(r) => {
                if let Some(current_edit) = &mut current_edit {
                    match current_edit.diff_type {
                        DiffType::Add | DiffType::Replace => {
                            current_edit.added_char_sequence.push(*r);
                        }
                        DiffType::Delete => {
                            // We detect that we actually have a "replace" edit and not a "delete" edit
                            current_edit.diff_type = DiffType::Replace;
                            current_edit.added_char_sequence.push(*r);
                        }
                    }
                } else {
                    current_edit = Some(TextDiff {
                        diff_type: DiffType::Add,
                        added_char_sequence: vec![*r],
                        removed_char_sequence: Vec::new(),
                        start_index: walk_index,
                    });
                }
            }
            diff::Result::Both(_, _) => {
                if let Some(current_edit) = current_edit.take() {
                    edits.push(current_edit);
                }
            }
        }
        walk_index += 1;
    }
    if let Some(current_edit) = current_edit.take() {
        edits.push(current_edit);
    }

    construct_InputEdits_from_detected_edits(old_string, new_string, &edits)
}

fn construct_InputEdits_from_detected_edits(
    old_string: &XcodeText,
    new_string: &XcodeText,
    detected_edits: &Vec<TextDiff>,
) -> Vec<InputEdit> {
    let mut input_edits: Vec<InputEdit> = Vec::new();
    for edit in detected_edits.iter() {
        let removed_char_count = edit.removed_char_sequence.len();
        let added_char_count = edit.added_char_sequence.len();

        let edit_range_before = match edit.diff_type {
            DiffType::Add => TextRange::new(edit.start_index, 0),
            DiffType::Delete => TextRange::new(edit.start_index, removed_char_count),
            DiffType::Replace => TextRange::new(edit.start_index, removed_char_count),
        };
        let edit_range_after = TextRange::new(edit.start_index, added_char_count);

        if let (Some(old_pts), Some(new_pts), Some(start_position)) = (
            edit_range_before.as_StartEndTSPoint(&old_string),
            edit_range_after.as_StartEndTSPoint(&new_string),
            TextPosition::from_TextIndex(old_string, edit.start_index),
        ) {
            input_edits.push(InputEdit {
                start_byte: edit.start_index * 2,
                old_end_byte: (edit.start_index + removed_char_count) * 2,
                new_end_byte: (edit.start_index + added_char_count) * 2,
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
