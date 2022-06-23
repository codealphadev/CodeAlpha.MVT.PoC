use accessibility::{AXAttribute, AXUIElement, AXUIElementAttributes, AXValue};
use cocoa::appkit::CGPoint;
use core_foundation::{base::CFRange, number::CFNumber};
use core_graphics_types::geometry::{CGRect, CGSize};

use crate::ax_interaction::{
    derive_xcode_textarea_dimensions, focused_uielement_of_app, window_ui_element_from_hash,
};

use super::{types::CharRange, MatchRectangle};

pub fn calc_match_rects_for_wrapped_range(
    wrapped_lines_count: usize,
    match_range: &CharRange,
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
        if let Ok((editor_origin, editor_size)) =
            derive_xcode_textarea_dimensions(&textarea_ui_element)
        {
            let first_line_rect = CGRect {
                origin: first_char_bounds.origin,
                size: CGSize {
                    width: editor_origin.x + editor_size.width - first_char_bounds.origin.x,
                    height: first_char_bounds.size.height,
                },
            };

            let last_line_rect = CGRect {
                origin: CGPoint {
                    x: editor_origin.x,
                    y: last_char_bounds.origin.y,
                },
                size: CGSize {
                    width: last_char_bounds.origin.x - editor_origin.x
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
                        println!("larger_than_two: {}", larger_than_two);
                        println!("i: {}", i);
                        let inbetween_line_rect = CGRect {
                            origin: CGPoint {
                                x: editor_origin.x,
                                y: first_char_bounds.origin.y
                                    + first_char_bounds.size.height * i as f64,
                            },
                            size: CGSize {
                                width: editor_size.width,
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

/// > Given a CharRange, it returns a tuple of (is_wrapped, line_count)
///
/// Arguments:
///
/// * `range`: &CharRange
/// * `textarea_ui_element`: The AXUIElement of the textarea
///
/// Returns:
///
/// A tuple of bool and usize
pub fn is_text_of_line_wrapped(
    range: &CharRange,
    textarea_ui_element: &AXUIElement,
) -> Option<(bool, usize)> {
    // Get rectangle of CharRange from AX apis
    if let Some(line_match_rect) = get_bounds_of_CharRange(range, textarea_ui_element) {
        // Calculate across how many line the CharRange is extends
        let line_count =
            calc_line_count_of_char_range(range, &line_match_rect, textarea_ui_element);
        if line_count > 1 {
            Some((true, line_count))
        } else if line_count == 1 {
            Some((false, line_count))
        } else {
            assert!(false, "Case line_count = 0 should never happen");
            None
        }
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
    line_match_range: &CharRange,
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
    range_index: i64,
    textarea_ui_element: &AXUIElement,
) -> Option<i64> {
    if let Ok(line_number) = textarea_ui_element.parameterized_attribute(
        &AXAttribute::line_for_index(),
        &CFNumber::from(range_index as i64),
    ) {
        line_number.to_i64()
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
/// A CharRange representing the character range of that line.
pub fn get_char_range_of_line(
    line_number: i64,
    textarea_ui_element: &AXUIElement,
) -> Option<CharRange> {
    if let Ok(line_char_range_as_axval) = textarea_ui_element.parameterized_attribute(
        &AXAttribute::range_for_line(),
        &CFNumber::from(line_number as i64),
    ) {
        if let Ok(line_char_CFRange) = line_char_range_as_axval.get_value::<CFRange>() {
            // The character range of a line in XCode always includes a line break character at the end.
            // Even at the end of the file. We need to remove that character from the character range.
            Some(CharRange {
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
        origin: tauri::LogicalPosition {
            x: cgrect.origin.x as f64,
            y: cgrect.origin.y as f64,
        },
        size: tauri::LogicalSize {
            width: cgrect.size.width as f64,
            height: cgrect.size.height as f64,
        },
    }
}

pub fn get_bounds_of_first_char_in_range(
    range: &CharRange,
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
    range: &CharRange,
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

pub fn get_bounds_of_CharRange(
    range: &CharRange,
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
        return Some(bounds_as_axval.get_value::<CGRect>().unwrap());
    }

    None
}
