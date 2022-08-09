use tree_sitter::InputEdit;

use crate::core_engine::rules::TextRange;
use diff;

#[derive(PartialEq, Clone, Debug)]
pub enum DiffType {
    Add,
    Delete,
    Replace,
}

#[derive(Clone, Debug)]
pub struct TextDiff {
    pub diff_type: DiffType,
    pub added_char_sequence: Vec<char>,
    pub removed_char_sequence: Vec<char>,
    pub start_index: usize,
    pub start_bytes: usize,
}

pub fn detect_input_edits(old_string: &String, new_string: &String) -> Vec<InputEdit> {
    // println!("old_string: {}", old_string);
    // println!("new_string: {}", new_string);

    let mut edits: Vec<TextDiff> = Vec::new();
    let mut detected_edit_option: Option<TextDiff> = None;
    let mut walk_index = 0;
    let mut bytes_counter = 0;
    for diff in diff::chars(&old_string, &new_string) {
        match diff {
            diff::Result::Left(l) => {
                // detect edit
                if let Some(detected_edit) = &mut detected_edit_option {
                    // Check if the previous edit is finished
                    if detected_edit.diff_type == DiffType::Add
                        || detected_edit.diff_type == DiffType::Replace
                    {
                        // Because both, 'Add' and 'Replace', only get characters added and not removed.
                        // The reason why 'Replace' only gets characters added is a) we always first remove and then add chars
                        // and b) a 'Replace' is only detected if after the removal of consecutive chars there is a add.
                        // Save the edit
                        edits.push(detected_edit.clone());

                        // Begin a new 'Delete' edit
                        detected_edit.diff_type = DiffType::Delete;
                        detected_edit.added_char_sequence.clear();
                        detected_edit.removed_char_sequence.clear();
                        detected_edit.start_index = walk_index;
                        detected_edit.start_bytes = bytes_counter;
                        detected_edit.removed_char_sequence.push(l);
                    } else if detected_edit.diff_type == DiffType::Delete {
                        detected_edit.removed_char_sequence.push(l);
                    }
                } else {
                    detected_edit_option = Some(TextDiff {
                        diff_type: DiffType::Delete,
                        added_char_sequence: Vec::new(),
                        removed_char_sequence: vec![l],
                        start_index: walk_index,
                        start_bytes: bytes_counter,
                    });
                }

                // count bytes
                bytes_counter += char_bytes(l);

                // print!("|-{}|", l)
            }
            diff::Result::Both(l, _) => {
                // detect edit
                // if we end up at a char which is neither added nor removed we wrap up the current edit
                if let Some(detected_edit) = &mut detected_edit_option {
                    edits.push(detected_edit.clone());
                    detected_edit_option = None;
                }

                // count bytes
                bytes_counter += char_bytes(l);

                // print!("{}", l)
            }
            diff::Result::Right(r) => {
                // detect edit
                if let Some(detected_edit) = &mut detected_edit_option {
                    // Check if the previous edit is finished
                    if detected_edit.diff_type == DiffType::Add
                        || detected_edit.diff_type == DiffType::Replace
                    {
                        detected_edit.added_char_sequence.push(r);
                    } else if detected_edit.diff_type == DiffType::Delete {
                        // We detect that we actually have a "replace" edit and not a "delete" edit
                        detected_edit.diff_type = DiffType::Replace;
                        detected_edit.added_char_sequence.push(r);
                    }
                } else {
                    detected_edit_option = Some(TextDiff {
                        diff_type: DiffType::Add,
                        added_char_sequence: vec![r],
                        removed_char_sequence: Vec::new(),
                        start_index: walk_index,
                        start_bytes: bytes_counter,
                    });
                }

                // count bytes
                bytes_counter += char_bytes(r);

                // print!("|+{}|", r)
            }
        }

        walk_index += 1;
    }

    // Check, if there is an unfinished edit after walking the whole string
    if let Some(detected_edit) = &mut detected_edit_option {
        edits.push(detected_edit.clone());
    }

    construct_InputEdits_from_detected_edits(old_string, new_string, &edits)
}

fn construct_InputEdits_from_detected_edits(
    old_string: &String,
    new_string: &String,
    detected_edits: &Vec<TextDiff>,
) -> Vec<InputEdit> {
    let mut input_edits: Vec<InputEdit> = Vec::new();
    for edit in detected_edits.iter() {
        let removed_char_count = edit.removed_char_sequence.len();
        let removed_bytes: usize = edit
            .removed_char_sequence
            .iter()
            .map(|c| char_bytes(*c))
            .sum();

        let added_char_count = edit.added_char_sequence.len();
        let added_bytes: usize = edit
            .added_char_sequence
            .iter()
            .map(|c| char_bytes(*c))
            .sum();

        let edit_range_before = match edit.diff_type {
            DiffType::Add => TextRange::new(edit.start_index, 0),
            DiffType::Delete => TextRange::new(edit.start_index, removed_char_count),
            DiffType::Replace => TextRange::new(edit.start_index, removed_char_count),
        };

        let edit_range_after = TextRange::new(edit.start_index, added_char_count);
        if let (Some(old_pts), Some(new_pts)) = (
            edit_range_before.as_StartEndTSPoint(old_string),
            edit_range_after.as_StartEndTSPoint(new_string),
        ) {
            input_edits.push(InputEdit {
                start_byte: edit.start_bytes,
                old_end_byte: edit.start_bytes + removed_bytes,
                new_end_byte: edit.start_bytes + added_bytes,
                start_position: old_pts.0,
                old_end_position: old_pts.1,
                new_end_position: new_pts.1,
            });
        }
    }

    // println!("{:#?}", detected_edits);
    // println!("===");
    // println!("");
    input_edits
}

fn char_bytes(c: char) -> usize {
    let size = c.len_utf8();
    if size == 3 {
        print!(
            "Warning: {} is 3 bytes; we didn't check support for such chars yet.",
            c
        );
    }

    size
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_detect_input_edits_remove_at_end_of_line() {
        let old_string = "abcdef";
        let new_string = "abcd";
        let input_edits = detect_input_edits(&old_string.to_string(), &new_string.to_string());
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, 4);
        assert_eq!(input_edits[0].old_end_byte, 6);
        assert_eq!(input_edits[0].new_end_byte, 4);
        assert_eq!(input_edits[0].start_position.row, 0);
        assert_eq!(input_edits[0].old_end_position.row, 0);
        assert_eq!(input_edits[0].new_end_position.row, 0);
        assert_eq!(input_edits[0].start_position.column, 4);
        assert_eq!(input_edits[0].old_end_position.column, 6);
        assert_eq!(input_edits[0].new_end_position.column, 4);
    }

    #[test]
    fn test_detect_input_edits_remove_with_emoji() {
        let old_string = "abcdef😊a";
        let new_string = "abcd";
        let input_edits = detect_input_edits(&old_string.to_string(), &new_string.to_string());
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, 4);
        assert_eq!(input_edits[0].old_end_byte, 8 + 3); // 3 additional bytes for the emoji
        assert_eq!(input_edits[0].new_end_byte, 4);
        assert_eq!(input_edits[0].start_position.row, 0);
        assert_eq!(input_edits[0].old_end_position.row, 0);
        assert_eq!(input_edits[0].new_end_position.row, 0);
        assert_eq!(input_edits[0].start_position.column, 4);
        assert_eq!(input_edits[0].old_end_position.column, 8);
        assert_eq!(input_edits[0].new_end_position.column, 4);
    }

    #[test]
    fn test_detect_input_edits_remove_with_emoji_end() {
        let old_string = "abcdef😊";
        let new_string = "abcd";
        let input_edits = detect_input_edits(&old_string.to_string(), &new_string.to_string());
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, 4);
        assert_eq!(input_edits[0].old_end_byte, 7 + 3); // 3 additional bytes for the emoji
        assert_eq!(input_edits[0].new_end_byte, 4);
        assert_eq!(input_edits[0].start_position.row, 0);
        assert_eq!(input_edits[0].old_end_position.row, 0);
        assert_eq!(input_edits[0].new_end_position.row, 0);
        assert_eq!(input_edits[0].start_position.column, 4);
        assert_eq!(input_edits[0].old_end_position.column, 7);
        assert_eq!(input_edits[0].new_end_position.column, 4);
    }

    #[test]
    fn test_detect_input_edits_add_with_emoji() {
        let old_string = "abcdef😊a";
        let new_string = "abcdef😊aBC";
        let input_edits = detect_input_edits(&old_string.to_string(), &new_string.to_string());
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, 8 + 3); // 3 additional bytes for the emoji
        assert_eq!(input_edits[0].old_end_byte, 8 + 3); // 3 additional bytes for the emoji
        assert_eq!(input_edits[0].new_end_byte, 10 + 3); // 3 additional bytes for the emoji
        assert_eq!(input_edits[0].start_position.row, 0);
        assert_eq!(input_edits[0].old_end_position.row, 0);
        assert_eq!(input_edits[0].new_end_position.row, 0);
        assert_eq!(input_edits[0].start_position.column, 8);
        assert_eq!(input_edits[0].old_end_position.column, 8);
        assert_eq!(input_edits[0].new_end_position.column, 10);
    }

    #[test]
    fn test_detect_input_edits_replace_with_emoji() {
        let old_string = "abc😊aBCd";
        let new_string = "abc😊aXXYYYd";
        let input_edits = detect_input_edits(&old_string.to_string(), &new_string.to_string());
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, 5 + 3); // 3 additional bytes for the emoji
        assert_eq!(input_edits[0].old_end_byte, 7 + 3); // 3 additional bytes for the emoji
        assert_eq!(input_edits[0].new_end_byte, 10 + 3); // 3 additional bytes for the emoji
        assert_eq!(input_edits[0].start_position.row, 0);
        assert_eq!(input_edits[0].old_end_position.row, 0);
        assert_eq!(input_edits[0].new_end_position.row, 0);
        assert_eq!(input_edits[0].start_position.column, 5);
        assert_eq!(input_edits[0].old_end_position.column, 7);
        assert_eq!(input_edits[0].new_end_position.column, 10);
    }

    #[test]
    fn test_detect_input_edits_replace_beginning_of_file() {
        let old_string = "let x = 1; console.log(x);";
        let new_string = "const x = 1; console.log(x);";
        let input_edits = detect_input_edits(&old_string.to_string(), &new_string.to_string());

        println!("{:#?}", &input_edits);
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, 0);
        assert_eq!(input_edits[0].old_end_byte, 2);
        assert_eq!(input_edits[0].new_end_byte, 4);
        assert_eq!(input_edits[0].start_position.row, 0);
        assert_eq!(input_edits[0].old_end_position.row, 0);
        assert_eq!(input_edits[0].new_end_position.row, 0);
        assert_eq!(input_edits[0].start_position.column, 0);
        assert_eq!(input_edits[0].old_end_position.column, 2);
        assert_eq!(input_edits[0].new_end_position.column, 4);
    }

    #[test]
    fn test_detect_input_edits_replace_in_middle_of_line() {
        let old_string = "fn test(XXX) {}";
        let new_string = "fn test(a: u32) {}";
        let input_edits = detect_input_edits(&old_string.to_string(), &new_string.to_string());

        println!("{:#?}", &input_edits);
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, 8);
        assert_eq!(input_edits[0].old_end_byte, 11);
        assert_eq!(input_edits[0].new_end_byte, 14);
        assert_eq!(input_edits[0].start_position.row, 0);
        assert_eq!(input_edits[0].old_end_position.row, 0);
        assert_eq!(input_edits[0].new_end_position.row, 0);
        assert_eq!(input_edits[0].start_position.column, 8);
        assert_eq!(input_edits[0].old_end_position.column, 11);
        assert_eq!(input_edits[0].new_end_position.column, 14);
    }

    #[test]
    fn test_detect_input_edits_replace_whole_line() {
        let old_string = "fn test0(a: u32) {}\n//row\nfn test2(a: u32) {}";
        let new_string = "fn test0(a: u32) {}\nfn test1(a: u32) {}\nfn test2(a: u32) {}";
        let input_edits = detect_input_edits(&old_string.to_string(), &new_string.to_string());

        println!("{:#?}", &input_edits);
        assert_eq!(input_edits.len(), 1);
        assert_eq!(input_edits[0].start_byte, 20);
        assert_eq!(input_edits[0].old_end_byte, 25);
        assert_eq!(input_edits[0].new_end_byte, 39);
        assert_eq!(input_edits[0].start_position.row, 1);
        assert_eq!(input_edits[0].old_end_position.row, 1);
        assert_eq!(input_edits[0].new_end_position.row, 1);
        assert_eq!(input_edits[0].start_position.column, 0);
        assert_eq!(input_edits[0].old_end_position.column, 5);
        assert_eq!(input_edits[0].new_end_position.column, 19);
    }
}
