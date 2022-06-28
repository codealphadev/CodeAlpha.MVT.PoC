#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A position in a multi-line text document, in terms of rows and columns.
/// Rows and columns are zero-based.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct TextPosition {
    pub row: usize,
    pub column: usize,
}

/// A position in a multi-line text document, in terms of index and length.
/// index is zero-based.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct TextRange {
    pub index: usize,
    pub length: usize,
}

pub fn TextRange_from_StartEndIndex(start_index: usize, end_index: usize) -> TextRange {
    TextRange {
        index: start_index,
        length: end_index - start_index,
    }
}

pub fn TextRange_from_StartEndPosition(
    text: &String,
    start_position: &TextPosition,
    end_position: &TextPosition,
) -> Option<TextRange> {
    let mut index: Option<usize> = None;
    let mut length: Option<usize> = None;

    let mut char_count = 0;
    let mut line_number = 0;
    while let Some(line) = text.lines().next() {
        if line_number == start_position.row {
            while let Some((col, _)) = line.char_indices().next() {
                if start_position.column == col {
                    index = Some(char_count);
                }
                char_count += 1;
            }
            line_number += 1;
        }

        if line_number == end_position.row {
            while let Some((col, _)) = line.char_indices().next() {
                if end_position.column == col {
                    length = Some(char_count - index.unwrap());
                }
                char_count += 1;
            }
            line_number += 1;
        }
    }

    if let (Some(index), Some(length)) = (index, length) {
        Some(TextRange { index, length })
    } else {
        None
    }
}

pub fn StartEndIndex_from_TextRange(char_range: &TextRange) -> (usize, usize) {
    (char_range.index, char_range.index + char_range.length)
}

pub fn StartEndIndex_from_StartEndPosition(
    text: &String,
    start_position: &TextPosition,
    end_position: &TextPosition,
) -> Option<(usize, usize)> {
    if let Some(char_range) = TextRange_from_StartEndPosition(text, start_position, end_position) {
        Some(StartEndIndex_from_TextRange(&char_range))
    } else {
        None
    }
}

pub fn StartEndPosition_from_TextRange(
    text: &String,
    char_range: &TextRange,
) -> Option<(TextPosition, TextPosition)> {
    let (start_index, end_index) = StartEndIndex_from_TextRange(&char_range);

    if let Some((start_position, end_position)) =
        StartEndPosition_from_StartEndIndex(text, start_index, end_index)
    {
        Some((start_position, end_position))
    } else {
        None
    }
}

pub fn StartEndPosition_from_StartEndIndex(
    text: &String,
    start_index: usize,
    end_index: usize,
) -> Option<(TextPosition, TextPosition)> {
    let mut start_position: Option<TextPosition> = None;
    let mut end_position: Option<TextPosition> = None;

    let mut char_count = 0;
    let mut line_number = 0;
    while let Some(line) = text.lines().next() {
        while let Some((col, _)) = line.char_indices().next() {
            if start_index == char_count {
                start_position = Some(TextPosition {
                    row: line_number,
                    column: col,
                });
            }

            if end_index == char_count {
                end_position = Some(TextPosition {
                    row: line_number,
                    column: col,
                });
            }
            char_count += 1;
        }
        line_number += 1;
    }

    if let (Some(start_position), Some(end_position)) = (start_position, end_position) {
        Some((start_position, end_position))
    } else {
        None
    }
}
