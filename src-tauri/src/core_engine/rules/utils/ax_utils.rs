use accessibility::{AXAttribute, AXUIElement, AXValue};
use cocoa::appkit::CGPoint;
use core_foundation::{base::CFRange, number::CFNumber};
use core_graphics_types::geometry::{CGRect, CGSize};
use std::convert::TryFrom;

use crate::{
    ax_interaction::{get_viewport_frame, GetVia},
    core_engine::utils::{TextRange, XcodeText},
    utils::geometry::{LogicalPosition, LogicalSize},
};

use super::{
    types::{LineMatch, MatchRange},
    MatchRectangle,
};

pub fn calc_rectangles_and_line_matches(
    match_range: &MatchRange,
    textarea_ui_element: &AXUIElement,
) -> (Vec<MatchRectangle>, Vec<LineMatch>) {
    let mut line_match_ranges: Vec<MatchRange> = Vec::new();

    // 2. Break up match range into individual matches that only span one line in the editor
    let mut current_match_index = match_range.range.index;
    while let Some(line_number) =
        get_line_number_for_range_index(current_match_index, &textarea_ui_element)
    {
        if let Some(current_line_range) = get_text_range_of_line(line_number, &textarea_ui_element)
        {
            // Check if the current line range is within the match range.
            if !current_line_range.includes_index(current_match_index)
                || !match_range.range.includes_index(current_match_index)
            {
                break;
            }

            let matched_char_range = TextRange {
                index: current_match_index,
                length: std::cmp::min(
                    current_line_range.length - (current_match_index - current_line_range.index),
                    match_range.range.length - (current_match_index - match_range.range.index),
                ),
            };

            let mut substr = XcodeText::new_empty();
            for (i, c) in match_range.string.iter().enumerate() {
                if i >= matched_char_range.index - match_range.range.index
                    && i < (matched_char_range.index - match_range.range.index)
                        + matched_char_range.length
                {
                    substr.push(*c);
                }
            }

            let line_match_range = MatchRange {
                string: substr,
                range: matched_char_range,
            };

            // Add +1 because current_line_range got its last char removed because it is always a line break character '\n'.
            // If we would not remove it, the calculated rectangles would stretch the the line to the end of the line.
            current_match_index = current_match_index + line_match_range.range.length + 1;
            line_match_ranges.push(line_match_range);

            if current_match_index >= match_range.range.index + match_range.range.length {
                break;
            }
        }
    }

    // 3. Calculate rectangles for each line match range; checking if they are wrapped, potentially adding multiple rectangles
    let mut rule_match_rectangles: Vec<MatchRectangle> = Vec::new();
    let mut line_matches: Vec<LineMatch> = Vec::new();
    for line_match_range in line_match_ranges {
        // Check if line_match_range actually wraps into multiple lines
        // due to activated 'wrap lines' in XCode (default is on)
        if let Some((range_is_wrapping, wrapped_line_number)) =
            is_text_of_range_wrapped(&line_match_range.range, &textarea_ui_element)
        {
            if !range_is_wrapping {
                if let Some(line_match_rect) =
                    get_bounds_of_TextRange(&line_match_range.range, &textarea_ui_element)
                {
                    rule_match_rectangles.push(line_match_rect.clone());
                    line_matches.push((line_match_range, vec![line_match_rect]));
                }
            } else {
                let line_match_rectangles = calc_match_rects_for_wrapped_range(
                    wrapped_line_number,
                    &line_match_range.range,
                    &textarea_ui_element,
                );

                rule_match_rectangles.extend(calc_match_rects_for_wrapped_range(
                    wrapped_line_number,
                    &line_match_range.range,
                    &textarea_ui_element,
                ));
                line_matches.push((line_match_range, line_match_rectangles));
            }
        }
    }

    (rule_match_rectangles, line_matches)
}

pub fn calc_match_rects_for_wrapped_range(
    wrapped_lines_count: usize,
    match_range: &TextRange,
    textarea_ui_element: &AXUIElement,
) -> Vec<MatchRectangle> {
    if wrapped_lines_count == 1 || wrapped_lines_count == 0 {
        assert!(
            false,
            "We should not be here, wrapped_lines_count should be > 1"
        );
    }

    if let (Some(first_char_bounds), Some(last_char_bounds)) = (
        get_bounds_of_first_char_in_range(&match_range, &textarea_ui_element),
        get_bounds_of_last_char_in_range(&match_range, &textarea_ui_element),
    ) {
        // Determine editor textarea's horizontal extent
        if let Ok(viewport_frame) = get_viewport_frame(&GetVia::Current) {
            let first_line_rect = CGRect {
                origin: first_char_bounds.origin,
                size: CGSize {
                    width: viewport_frame.origin.x + viewport_frame.size.width
                        - first_char_bounds.origin.x,
                    height: first_char_bounds.size.height,
                },
            };

            let last_line_rect = CGRect {
                origin: CGPoint {
                    x: viewport_frame.origin.x,
                    y: last_char_bounds.origin.y,
                },
                size: CGSize {
                    width: last_char_bounds.origin.x - viewport_frame.origin.x
                        + last_char_bounds.size.width,
                    height: last_char_bounds.size.height,
                },
            };

            let first_line_match_rectangle = get_MatchRect_from_CGRect(&first_line_rect);
            let last_line_match_rectangle = get_MatchRect_from_CGRect(&last_line_rect);

            match wrapped_lines_count {
                2 => {
                    // Case A: if wrapped_lines_count = 2
                    // ==================================
                    // - First line gets a rectangle from the FIRST character rectangle of the
                    //   match string to the RIGHT end of the text area
                    // - Last line gets a rectangle from the LAST character rectangle of the
                    //   match string to the LEFT end of the text area
                    vec![first_line_match_rectangle, last_line_match_rectangle]
                }
                larger_than_two => {
                    // Case B: if wrapped_lines_count > 2
                    // ==================================
                    // - Same as Case A
                    // - All lines inbetween first and last line get a rectangle stretching
                    //   from the LEFT end of the text area to the RIGHT end of the text area

                    let mut inbetween_line_rectangles = Vec::<MatchRectangle>::new();

                    // Minus 1 because rectangles for first and last line are already added
                    // E.g. if wrapped_lines_count = 3, we need to add 1 inbetween-rectangle
                    for i in 1..larger_than_two - 1 {
                        let inbetween_line_rect = CGRect {
                            origin: CGPoint {
                                x: viewport_frame.origin.x,
                                y: first_char_bounds.origin.y
                                    + first_char_bounds.size.height * i as f64,
                            },
                            size: CGSize {
                                width: viewport_frame.size.width,
                                height: first_char_bounds.size.height,
                            },
                        };

                        inbetween_line_rectangles
                            .push(get_MatchRect_from_CGRect(&inbetween_line_rect));
                    }

                    vec![
                        vec![first_line_match_rectangle],
                        inbetween_line_rectangles,
                        vec![last_line_match_rectangle],
                    ]
                    .into_iter()
                    .flatten()
                    .collect()
                }
            }
        } else {
            assert!(false, "Case A could not get dimensions of editor textarea");
            vec![]
        }
    } else {
        assert!(
            false,
            "Case A could not get CGRects for first and last character"
        );
        vec![]
    }
}

/// > Given a TextRange, it returns a tuple of (is_wrapped, line_count)
///
/// Arguments:
///
/// * `range`: &TextRange
/// * `textarea_ui_element`: The AXUIElement of the textarea
///
/// Returns:
///
/// A tuple of bool and usize
pub fn is_text_of_range_wrapped(
    range: &TextRange,
    textarea_ui_element: &AXUIElement,
) -> Option<(bool, usize)> {
    // Get rectangle of TextRange from AX apis
    if let Some(line_match_rect) = get_bounds_of_TextRange(range, textarea_ui_element) {
        // Calculate across how many line the TextRange is extends
        let line_count =
            calc_line_count_of_char_range(range, &line_match_rect, textarea_ui_element);
        if line_count > 1 {
            Some((true, line_count))
        } else if line_count == 1 {
            Some((false, line_count))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn is_text_of_line_wrapped(
    line: usize,
    textarea_ui_element: &AXUIElement,
) -> Option<(bool, usize)> {
    if let Some(line_text_range) = get_text_range_of_line(line, &textarea_ui_element) {
        is_text_of_range_wrapped(&line_text_range, &textarea_ui_element)
    } else {
        None
    }
}

/// It calculates the number of lines in a match range by dividing the height of the match rectangle by
/// the height of a single line
///
/// Arguments:
///
/// * `line_match_range`: The range of characters that the line match is on.
/// * `line_match_rectangle`: The rectangle that contains the line of text that contains the match.
/// * `textarea_ui_element`: The AXUIElement of the textarea.
///
/// Returns:
///
/// The number of lines in the match range.
pub fn calc_line_count_of_char_range(
    line_match_range: &TextRange,
    line_match_rectangle: &MatchRectangle,
    textarea_ui_element: &AXUIElement,
) -> usize {
    if let Some(line_height_cgrect) = get_bounds_of_CFRange(
        &CFRange {
            location: line_match_range.index as isize,
            length: 0,
        },
        textarea_ui_element,
    ) {
        (line_match_rectangle.size.height / line_height_cgrect.size.height).round() as usize
    } else {
        1
    }
}

/// It takes a range index and a textarea UI element and returns the line number for that range index
///
/// Arguments:
///
/// * `range_index`: The index of the character in match range.
/// * `textarea_ui_element`: The AXUIElement of the textarea.
///
/// Returns:
///
/// The line number for the given range index.
pub fn get_line_number_for_range_index(
    range_index: usize,
    textarea_ui_element: &AXUIElement,
) -> Option<usize> {
    if let Ok(line_number) = textarea_ui_element.parameterized_attribute(
        &AXAttribute::line_for_index(),
        &CFNumber::from(range_index as i64),
    ) {
        if let Some(line_number) = line_number.to_i64() {
            if let Ok(line_number) = usize::try_from(line_number) {
                return Some(line_number);
            }
        }
        None
    } else {
        None
    }
}

/// It takes a line number and a textarea UI element, and returns the character range of that line.
/// The character range of a line in XCode always includes a line break character at the end. Even
/// at the end of the file. We need to remove that character from the character range.
///
/// Arguments:
///
/// * `line_number`: The line number of the line you want to get the character range of.
/// * `textarea_ui_element`: The AXUIElement of the textarea.
///
/// Returns:
///
/// A TextRange representing the character range of that line.
pub fn get_text_range_of_line(
    line_number: usize,
    textarea_ui_element: &AXUIElement,
) -> Option<TextRange> {
    if let Ok(line_char_range_as_axval) = textarea_ui_element.parameterized_attribute(
        &AXAttribute::range_for_line(),
        &CFNumber::from(line_number as i64),
    ) {
        if let Ok(line_char_CFRange) = line_char_range_as_axval.get_value::<CFRange>() {
            // The character range of a line in XCode always includes a line break character at the end.
            // Even at the end of the file. We need to remove that character from the character range.
            Some(TextRange {
                index: line_char_CFRange.location as usize,
                length: (line_char_CFRange.length - 1) as usize,
            })
        } else {
            None
        }
    } else {
        None
    }
}

/// It takes a CGRect and returns a MatchRectangle
///
/// Arguments:
///
/// * `cgrect`: The CGRect that we want to convert to a MatchRectangle
///
/// Returns:
///
/// A MatchRectangle
pub fn get_MatchRect_from_CGRect(cgrect: &CGRect) -> MatchRectangle {
    MatchRectangle {
        origin: LogicalPosition::from_CGPoint(&cgrect.origin),
        size: LogicalSize::from_CGSize(&cgrect.size),
    }
}

pub fn get_bounds_of_first_char_in_range(
    range: &TextRange,
    textarea_ui_element: &AXUIElement,
) -> Option<CGRect> {
    get_bounds_of_CFRange(
        &CFRange {
            location: range.index as isize,
            length: 1,
        },
        textarea_ui_element,
    )
}

pub fn get_bounds_of_last_char_in_range(
    range: &TextRange,
    textarea_ui_element: &AXUIElement,
) -> Option<CGRect> {
    get_bounds_of_CFRange(
        &CFRange {
            location: (range.index + range.length - 1) as isize,
            length: 1,
        },
        textarea_ui_element,
    )
}

pub fn get_bounds_of_TextRange(
    range: &TextRange,
    textarea_ui_element: &AXUIElement,
) -> Option<MatchRectangle> {
    let cf_range_val = CFRange {
        location: range.index as isize,
        length: range.length as isize,
    };

    if let Some(cgrect) = get_bounds_of_CFRange(&cf_range_val, textarea_ui_element) {
        Some(get_MatchRect_from_CGRect(&cgrect))
    } else {
        None
    }
}

pub fn get_bounds_of_CFRange(range: &CFRange, textarea_ui_element: &AXUIElement) -> Option<CGRect> {
    let axval_from_cfrange = AXValue::from_CFRange(*range).unwrap();

    if let Ok(bounds_as_axval) = textarea_ui_element
        .parameterized_attribute(&AXAttribute::bounds_for_range(), &axval_from_cfrange)
    {
        // Convert AXValue into valid CGRect
        let result = bounds_as_axval.get_value::<CGRect>().unwrap();
        // If off screen, it returns 0.0, 0.0 for size; treat this as no match
        if result.size.width == 0.0 && result.size.height == 0.0 {
            return None;
        }
        return Some(result);
    }

    None
}
