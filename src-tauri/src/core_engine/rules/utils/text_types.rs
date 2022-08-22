#![allow(dead_code)]

use crate::core_engine::utils::{TextPosition, TextRange, XcodeChar, XcodeText};

pub fn StartEndIndex_from_StartEndTextPosition(
    text: &XcodeText,
    start_position: &TextPosition,
    end_position: &TextPosition,
) -> Option<(usize, usize)> {
    if let Some(char_range) =
        TextRange::from_StartEndTextPosition(text, start_position, end_position)
    {
        Some(char_range.as_StartEndIndex())
    } else {
        None
    }
}

pub fn StartEndIndex_from_StartEndTSPoint(
    text: &XcodeText,
    start_point: &tree_sitter::Point,
    end_point: &tree_sitter::Point,
) -> Option<(usize, usize)> {
    StartEndIndex_from_StartEndTextPosition(
        text,
        &TextPosition::from_TSPoint(start_point),
        &TextPosition::from_TSPoint(end_point),
    )
}

pub fn StartEndTextPosition_from_StartEndIndex(
    text: &XcodeText,
    start_index: usize,
    end_index: usize,
) -> Option<(TextPosition, TextPosition)> {
    let range = TextRange::from_StartEndIndex(start_index, end_index);

    range.as_StartEndTextPosition(&text)
}

pub fn StartEndTextPosition_from_StartEndTSPoint(
    start_point: &tree_sitter::Point,
    end_point: &tree_sitter::Point,
) -> (TextPosition, TextPosition) {
    (
        TextPosition::from_TSPoint(start_point),
        TextPosition::from_TSPoint(end_point),
    )
}

pub fn StartEndTSPoint_from_StartEndIndex(
    text: &XcodeText,
    start_index: usize,
    end_index: usize,
) -> Option<(tree_sitter::Point, tree_sitter::Point)> {
    if let Some((start_pos, end_pos)) =
        StartEndTextPosition_from_StartEndIndex(&text, start_index, end_index)
    {
        Some((start_pos.as_TSPoint(), end_pos.as_TSPoint()))
    } else {
        None
    }
}

pub fn StartEndTSPoint_from_StartEndTextPosition(
    start_position: &TextPosition,
    end_position: &TextPosition,
) -> (tree_sitter::Point, tree_sitter::Point) {
    (
        TextPosition::as_TSPoint(start_position),
        TextPosition::as_TSPoint(end_position),
    )
}

#[cfg(test)]
mod tests_TextConversions {
    use crate::core_engine::{
        rules::utils::text_types::{
            StartEndIndex_from_StartEndTSPoint, StartEndIndex_from_StartEndTextPosition,
            TextPosition,
        },
        utils::XcodeText,
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn test_StartEndIndex_from_StartEndTextPosition_one_line() {
        let text = XcodeText::from_str("Hello, World!");
        //                |--->|  start index 0, end index 5
        let start_position = TextPosition::new(0, 0);
        let end_position = TextPosition::new(0, 5);
        let result = StartEndIndex_from_StartEndTextPosition(&text, &start_position, &end_position);

        assert_eq!(result.is_some(), true);

        let (start_index, end_index) = result.unwrap();

        assert_eq!(start_index, 0);
        assert_eq!(end_index, 5);
    }

    #[test]
    fn test_StartEndIndex_from_StartEndTextPosition_two_lines() {
        let text = XcodeText::from_str("Hello, World!\nHello, World!");
        //                |------------- ---->|  start index 0, end index 19
        let start_position = TextPosition::new(0, 0);
        let end_position = TextPosition::new(1, 5);
        let result = StartEndIndex_from_StartEndTextPosition(&text, &start_position, &end_position);

        assert_eq!(result.is_some(), true);

        let (start_index, end_index) = result.unwrap();

        assert_eq!(start_index, 0);
        assert_eq!(end_index, 19);
    }

    // Write test for StartEndIndex_from_StartEndTSPoint
    #[test]
    fn test_StartEndIndex_from_StartEndTSPoint_one_line() {
        let text = XcodeText::from_str("Hello, World!");
        let start_position = TextPosition::new(0, 0);
        let end_position = TextPosition::new(0, 5);
        let start_point = start_position.as_TSPoint();
        let end_point = end_position.as_TSPoint();
        let result = StartEndIndex_from_StartEndTSPoint(&text, &start_point, &end_point);

        assert_eq!(result.is_some(), true);

        let (start_index, end_index) = result.unwrap();

        assert_eq!(start_index, 0);
        assert_eq!(end_index, 5);
    }
}

pub fn get_index_of_next_row(index: usize, text: &XcodeText) -> Option<usize> {
    let mut i = 0;
    for c in text[index..].to_vec() {
        if c == '\n' as XcodeChar {
            return Some(index + i + 1);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests_Text {
    use crate::core_engine::{rules::get_index_of_next_row, utils::XcodeText};

    #[test]
    fn test_index_of_next_row() {
        // No new row in text
        assert_eq!(
            get_index_of_next_row(5, &(XcodeText::from_str("Hello, World!"))),
            None
        );

        // return index at new row and keep end index
        let text = XcodeText::from_str(
            "Hello,
      World!",
        );
        assert_eq!(get_index_of_next_row(5, &text), Some(7));

        // return index at new row and keep end index
        assert_eq!(
            get_index_of_next_row(5, &XcodeText::from_str("Hello test,\n Wor!ld!\n")),
            Some(12)
        );
    }
}
