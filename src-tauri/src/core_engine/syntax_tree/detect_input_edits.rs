use tree_sitter::InputEdit;

use crate::core_engine::rules::TextRange;

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
}

pub fn detect_input_edits(old_str: &String, new_str: &String) -> Vec<InputEdit> {
    extern crate diff;

    let mut edits: Vec<TextDiff> = Vec::new();
    let mut detected_edit_option: Option<TextDiff> = None;
    let mut walk_index = 0;
    for diff in diff::chars(&old_str, &new_str) {
        match diff {
            diff::Result::Left(l) => {
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
                    });
                }

                // print!("|-{}|", l)
            }
            diff::Result::Both(_, _) => {
                // if we end up at a char which is neither added nor removed we wrap up the current edit
                if let Some(detected_edit) = &mut detected_edit_option {
                    edits.push(detected_edit.clone());
                    detected_edit_option = None;
                }
                // print!("{}", l)
            }
            diff::Result::Right(r) => {
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
                    });
                }

                // print!("|+{}|", r)
            }
        }

        walk_index += 1;
    }

    // Check, if there is an unfinished edit after walking the whole string
    if let Some(detected_edit) = &mut detected_edit_option {
        edits.push(detected_edit.clone());
    }

    // Construct InputEdits from the detected edits
    let mut input_edits: Vec<InputEdit> = Vec::new();
    for edit in edits.iter() {
        let removed_char_count = edit.removed_char_sequence.len();
        let added_char_count = edit.added_char_sequence.len();

        let edit_range_before = match edit.diff_type {
            DiffType::Add => TextRange::new(edit.start_index, 0),
            DiffType::Delete => TextRange::new(edit.start_index, removed_char_count),
            DiffType::Replace => TextRange::new(edit.start_index, removed_char_count),
        };

        let edit_range_after = TextRange::new(edit.start_index, added_char_count);

        // println!("{:?}", edit_range_before);
        // println!("{:?}", edit_range_after);

        if let (Some(old_pts), Some(new_pts)) = (
            edit_range_before.as_StartEndTSPoint(old_str),
            edit_range_after.as_StartEndTSPoint(new_str),
        ) {
            input_edits.push(InputEdit {
                start_byte: edit.start_index,
                old_end_byte: edit_range_before.index + edit_range_before.length,
                new_end_byte: edit_range_after.index + edit_range_after.length,
                start_position: old_pts.0,
                old_end_position: old_pts.1,
                new_end_position: new_pts.1,
            });
        }
    }

    // println!("{:#?}", input_edits);
    // println!("===");
    // println!("");
    // println!("{:#?}", edits);
    // println!("===");
    // println!("");

    input_edits
}
