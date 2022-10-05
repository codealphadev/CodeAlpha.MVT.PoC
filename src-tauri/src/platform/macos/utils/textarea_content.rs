use accessibility::{AXAttribute, AXValue};
use core_foundation::{base::CFRange, number::CFNumber};
use core_graphics::geometry::CGRect;
use mockall::*;

use crate::{
    core_engine::{TextRange, XcodeText},
    utils::{
        geometry::{LogicalFrame, LogicalPosition, LogicalSize},
        rule_types::{LineMatch, MatchRange},
    },
};

use super::{get_textarea_uielement, internal::get_uielement_frame, GetVia, XcodeError};

#[automock]
pub trait AXTextareaContentUtils {
    fn get_bounds_for_TextRange(
        range: &TextRange,
        get_via: &GetVia,
    ) -> Result<LogicalFrame, XcodeError>;

    fn calc_line_count_of_char_range(
        line_match_range: &TextRange,
        line_match_rectangle: &LogicalFrame,
        get_via: &GetVia,
    ) -> Result<usize, XcodeError>;

    fn get_line_number_for_range_index(
        range_index: usize,
        get_via: &GetVia,
    ) -> Result<usize, XcodeError>;

    fn get_text_range_of_line(
        line_number: usize,
        get_via: &GetVia,
    ) -> Result<TextRange, XcodeError>;

    fn is_text_of_range_wrapped(
        range: &TextRange,
        get_via: &GetVia,
    ) -> Result<(bool, usize), XcodeError>;

    fn is_text_of_line_wrapped(lline: usize, get_via: &GetVia)
        -> Result<(bool, usize), XcodeError>;

    fn calc_match_rects_for_wrapped_range(
        wrapped_lines_count: usize,
        match_range: &TextRange,
        get_via: &GetVia,
    ) -> Result<Vec<LogicalFrame>, XcodeError>;

    fn split_match_by_lines(match_range: &MatchRange, get_via: &GetVia) -> Vec<MatchRange>;

    fn calc_rectangles_and_line_matches(
        match_range: &MatchRange,
        get_via: &GetVia,
    ) -> Result<(Vec<LogicalFrame>, Vec<LineMatch>), XcodeError>;
}

pub struct TextAreaContent {}

impl AXTextareaContentUtils for TextAreaContent {
    fn get_bounds_for_TextRange(
        range: &TextRange,
        get_via: &GetVia,
    ) -> Result<LogicalFrame, XcodeError> {
        let axval_from_cfrange = AXValue::from_CFRange(range.as_CFRange())
            .map_err(|err| XcodeError::AXError(err.into()))?;

        let textarea_uielement = get_textarea_uielement(get_via)?;

        match textarea_uielement
            .parameterized_attribute(&AXAttribute::bounds_for_range(), &axval_from_cfrange)
        {
            Ok(bounds_as_axval) => {
                let bounds_rectangle = bounds_as_axval
                    .get_value::<CGRect>()
                    .map_err(|err| XcodeError::AXError(err.into()))?;
                // If off screen, it returns 0.0, 0.0 for size - error.
                if bounds_rectangle.size.width == 0.0 && bounds_rectangle.size.height == 0.0 {
                    return Err(XcodeError::NotContainedVisibleTextRange);
                }
                return Ok(LogicalFrame::from_CGRect(&bounds_rectangle));
            }
            Err(err) => Err(XcodeError::AXError(err.into())),
        }
    }

    fn calc_line_count_of_char_range(
        line_match_range: &TextRange,
        line_match_rectangle: &LogicalFrame,
        get_via: &GetVia,
    ) -> Result<usize, XcodeError> {
        let line_frame = Self::get_bounds_for_TextRange(
            &TextRange {
                index: line_match_range.index,
                length: 0,
            },
            get_via,
        )?;

        Ok((line_match_rectangle.size.height / line_frame.size.height).round() as usize)
    }

    fn get_line_number_for_range_index(
        range_index: usize,
        get_via: &GetVia,
    ) -> Result<usize, XcodeError> {
        let textarea_uielement = get_textarea_uielement(get_via)?;

        match textarea_uielement.parameterized_attribute(
            &AXAttribute::line_for_index(),
            &CFNumber::from(range_index as i64),
        ) {
            Ok(line_number) => {
                if let Some(line_number) = line_number.to_i64() {
                    if let Ok(line_number) = usize::try_from(line_number) {
                        return Ok(line_number);
                    }
                }

                Err(XcodeError::AXError(accessibility::Error::NotFound.into()))
            }
            Err(err) => Err(XcodeError::AXError(err.into())),
        }
    }

    fn get_text_range_of_line(
        line_number: usize,
        get_via: &GetVia,
    ) -> Result<TextRange, XcodeError> {
        let textarea_uielement = get_textarea_uielement(get_via)?;

        match textarea_uielement.parameterized_attribute(
            &AXAttribute::range_for_line(),
            &CFNumber::from(line_number as i64),
        ) {
            Ok(line_char_range_as_axval) => {
                if let Ok(line_char_CFRange) = line_char_range_as_axval.get_value::<CFRange>() {
                    // The character range of a line in XCode always includes a line break character at the end.
                    // Even at the end of the file. We need to remove that character from the character range.
                    return Ok(TextRange {
                        index: line_char_CFRange.location as usize,
                        length: (line_char_CFRange.length - 1) as usize,
                    });
                }

                Err(XcodeError::AXError(accessibility::Error::NotFound.into()))
            }
            Err(err) => Err(XcodeError::AXError(err.into())),
        }
    }

    fn is_text_of_range_wrapped(
        range: &TextRange,
        get_via: &GetVia,
    ) -> Result<(bool, usize), XcodeError> {
        // Get rectangle of TextRange from AX apis
        let line_match_rect = Self::get_bounds_for_TextRange(range, get_via)?;

        // Calculate across how many line the TextRange is extends
        let line_count = Self::calc_line_count_of_char_range(range, &line_match_rect, get_via)?;

        if line_count > 1 {
            Ok((true, line_count))
        } else if line_count == 1 {
            Ok((false, line_count))
        } else {
            Err(XcodeError::ImplausibleDimensions)
        }
    }

    fn is_text_of_line_wrapped(line: usize, get_via: &GetVia) -> Result<(bool, usize), XcodeError> {
        let line_text_range = Self::get_text_range_of_line(line, &get_via)?;

        Self::is_text_of_range_wrapped(&line_text_range, &get_via)
    }

    fn calc_match_rects_for_wrapped_range(
        wrapped_lines_count: usize,
        match_range: &TextRange,
        get_via: &GetVia,
    ) -> Result<Vec<LogicalFrame>, XcodeError> {
        if wrapped_lines_count == 1 || wrapped_lines_count == 0 {
            return Err(XcodeError::CustomError(
                "We should not be here, wrapped_lines_count should be > 1".to_string(),
            ));
        }

        let bounds_first_char = Self::get_bounds_for_TextRange(
            &match_range
                .first_char_as_TextRange()
                .ok_or(XcodeError::CustomError("Invalid TextRange".to_string()))?,
            get_via,
        )?;

        let bounds_last_char = Self::get_bounds_for_TextRange(
            &match_range
                .last_char_as_TextRange()
                .ok_or(XcodeError::CustomError("Invalid TextRange".to_string()))?,
            get_via,
        )?;

        // Determine editor textarea's horizontal extent
        let viewport_frame = get_uielement_frame(&get_textarea_uielement(get_via)?)?;

        let first_line_rect = LogicalFrame {
            origin: bounds_first_char.origin,
            size: LogicalSize {
                width: viewport_frame.origin.x + viewport_frame.size.width
                    - bounds_first_char.origin.x,
                height: bounds_first_char.size.height,
            },
        };

        let last_line_rect = LogicalFrame {
            origin: LogicalPosition {
                x: viewport_frame.origin.x,
                y: bounds_last_char.origin.y,
            },
            size: LogicalSize {
                width: bounds_last_char.origin.x - viewport_frame.origin.x
                    + bounds_last_char.size.width,
                height: bounds_last_char.size.height,
            },
        };

        match wrapped_lines_count {
            2 => {
                // Case A: if wrapped_lines_count = 2
                // ==================================
                // - First line gets a rectangle from the FIRST character rectangle of the
                //   match string to the RIGHT end of the text area
                // - Last line gets a rectangle from the LAST character rectangle of the
                //   match string to the LEFT end of the text area
                Ok(vec![first_line_rect, last_line_rect])
            }
            larger_than_two => {
                // Case B: if wrapped_lines_count > 2
                // ==================================
                // - Same as Case A
                // - All lines inbetween first and last line get a rectangle stretching
                //   from the LEFT end of the text area to the RIGHT end of the text area

                let mut inbetween_line_rectangles = Vec::<LogicalFrame>::new();

                // Minus 1 because rectangles for first and last line are already added
                // E.g. if wrapped_lines_count = 3, we need to add 1 inbetween-rectangle
                for i in 1..larger_than_two - 1 {
                    let inbetween_line_rect = LogicalFrame {
                        origin: LogicalPosition {
                            x: viewport_frame.origin.x,
                            y: bounds_first_char.origin.y
                                + bounds_first_char.size.height * i as f64,
                        },
                        size: LogicalSize {
                            width: viewport_frame.size.width,
                            height: bounds_first_char.size.height,
                        },
                    };

                    inbetween_line_rectangles.push(inbetween_line_rect);
                }

                Ok(vec![
                    vec![first_line_rect],
                    inbetween_line_rectangles,
                    vec![last_line_rect],
                ]
                .into_iter()
                .flatten()
                .collect())
            }
        }
    }

    // Break up match range into individual matches that only span one line in the editor
    fn split_match_by_lines(match_range: &MatchRange, get_via: &GetVia) -> Vec<MatchRange> {
        let mut line_match_ranges: Vec<MatchRange> = Vec::new();

        let mut current_match_index = match_range.range.index;
        while let Ok(line_number) =
            Self::get_line_number_for_range_index(current_match_index, get_via)
        {
            if let Ok(current_line_range) = Self::get_text_range_of_line(line_number, get_via) {
                // Check if the current line range is within the match range.
                if !current_line_range.includes_index(current_match_index)
                    || !match_range.range.includes_index(current_match_index)
                {
                    break;
                }

                let matched_char_range = TextRange {
                    index: current_match_index,
                    length: std::cmp::min(
                        current_line_range.length
                            - (current_match_index - current_line_range.index),
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
        return line_match_ranges;
    }

    fn calc_rectangles_and_line_matches(
        match_range: &MatchRange,
        get_via: &GetVia,
    ) -> Result<(Vec<LogicalFrame>, Vec<LineMatch>), XcodeError> {
        let line_match_ranges: Vec<MatchRange> = Self::split_match_by_lines(match_range, get_via);

        // Calculate rectangles for each line match range; checking if they are wrapped, potentially adding multiple rectangles
        let mut rule_match_rectangles: Vec<LogicalFrame> = Vec::new();
        let mut line_matches: Vec<LineMatch> = Vec::new();
        for line_match_range in line_match_ranges {
            // Check if line_match_range actually wraps into multiple lines
            // due to activated 'wrap lines' in XCode (default is on)
            if let Ok((range_is_wrapping, wrapped_line_number)) =
                Self::is_text_of_range_wrapped(&line_match_range.range, get_via)
            {
                if !range_is_wrapping {
                    if let Ok(line_match_rect) =
                        Self::get_bounds_for_TextRange(&line_match_range.range, get_via)
                    {
                        rule_match_rectangles.push(line_match_rect.clone());
                        line_matches.push((line_match_range, vec![line_match_rect]));
                    }
                } else {
                    let line_match_rectangles = Self::calc_match_rects_for_wrapped_range(
                        wrapped_line_number,
                        &line_match_range.range,
                        get_via,
                    )?;

                    rule_match_rectangles.extend(Self::calc_match_rects_for_wrapped_range(
                        wrapped_line_number,
                        &line_match_range.range,
                        get_via,
                    )?);

                    line_matches.push((line_match_range, line_match_rectangles));
                }
            }
        }

        Ok((rule_match_rectangles, line_matches))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core_engine::TextRange,
        platform::macos::{GetVia, XcodeError},
        utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    };

    use super::{AXTextareaContentUtils, MockAXTextareaContentUtils};

    use lazy_static::lazy_static;
    use parking_lot::Mutex;

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn get_line_number_for_range_index() {
        let _m = MTX.lock();

        let ctx = MockAXTextareaContentUtils::get_bounds_for_TextRange_context();
        ctx.expect().returning(|_, _| {
            Ok(LogicalFrame {
                origin: LogicalPosition { x: 0., y: 0. },
                size: LogicalSize {
                    width: 0.,
                    height: 0.,
                },
            })
        });

        let expected: Result<LogicalFrame, XcodeError> = Ok(LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 0.,
                height: 0.,
            },
        });

        assert_eq!(
            expected.is_ok(),
            MockAXTextareaContentUtils::get_bounds_for_TextRange(
                &TextRange {
                    index: 0,
                    length: 0
                },
                &GetVia::Current
            )
            .is_ok()
        )
    }

    #[test]
    fn get_line_number_for_range_index2() {
        let _m = MTX.lock();

        let ctx = MockAXTextareaContentUtils::get_bounds_for_TextRange_context();
        ctx.expect().returning(|_, _| {
            Ok(LogicalFrame {
                origin: LogicalPosition { x: 0., y: 0. },
                size: LogicalSize {
                    width: 0.,
                    height: 0.,
                },
            })
        });

        let expected: Result<LogicalFrame, XcodeError> = Ok(LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 0.,
                height: 0.,
            },
        });

        assert_eq!(
            expected.is_ok(),
            MockAXTextareaContentUtils::get_bounds_for_TextRange(
                &TextRange {
                    index: 0,
                    length: 0
                },
                &GetVia::Current
            )
            .is_ok()
        )
    }
}
