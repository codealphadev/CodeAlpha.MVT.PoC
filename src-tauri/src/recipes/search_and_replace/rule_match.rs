use crate::ax_interaction::focused_uielement_of_app;

use super::utils::ax_utils::{
    calc_match_rects_for_wrapped_range, get_bounds_of_MatchRange, get_char_range_of_line,
    get_line_number_for_range_index, is_text_of_line_wrapped,
};

use super::utils::types::{MatchRange, MatchRectangle};

#[derive(Debug)]
pub struct RuleMatch {
    pub range: MatchRange,
    pub matched: String,
    pub rectangles: Vec<MatchRectangle>,
}

impl RuleMatch {
    pub fn new(range: MatchRange, matched: String) -> Self {
        Self {
            range,
            matched,
            rectangles: Vec::new(),
        }
    }

    pub fn update_rectangles(&mut self, editor_app_pid: i32) {
        // Get Editor TextArea UI Element
        if let Ok(editor_textarea_ui_element) = focused_uielement_of_app(editor_app_pid) {
            let mut line_match_ranges: Vec<MatchRange> = Vec::new();

            let mut current_match_index = dbg!(self.range.index);
            while let Some(line_number) = get_line_number_for_range_index(
                current_match_index as i64,
                &editor_textarea_ui_element,
            ) {
                // Get line length
                if let Some(current_line_range) = dbg!(get_char_range_of_line(
                    line_number,
                    &editor_textarea_ui_element
                )) {
                    let line_match_range = dbg!(MatchRange {
                        index: current_match_index,
                        length: std::cmp::min(
                            current_line_range.length
                                - (current_match_index - current_line_range.index),
                            self.range.length - (current_match_index - self.range.index),
                        ),
                    });

                    current_match_index = current_match_index + line_match_range.length;
                    line_match_ranges.push(line_match_range);

                    if current_match_index >= self.range.index + self.range.length {
                        break;
                    }
                }
            }

            dbg!(&line_match_ranges);

            let mut line_match_range_rectangles: Vec<MatchRectangle> = Vec::new();
            for line_match_range in line_match_ranges {
                // Check if line_match_range actually wraps into multiple lines
                // due to activated 'wrap lines' in XCode (default is on)
                if let Some((range_is_wrapping, wrapped_line_number)) =
                    is_text_of_line_wrapped(&line_match_range, &editor_textarea_ui_element)
                {
                    if !range_is_wrapping {
                        println!("Line is not wrapped");
                        if let Some(line_match_rect) =
                            get_bounds_of_MatchRange(&line_match_range, &editor_textarea_ui_element)
                        {
                            line_match_range_rectangles.push(line_match_rect);
                        }
                    } else {
                        println!("Line is wrapped");
                        line_match_range_rectangles.extend(calc_match_rects_for_wrapped_range(
                            wrapped_line_number,
                            &line_match_range,
                            &editor_textarea_ui_element,
                        ));
                    }
                }
            }

            dbg!(line_match_range_rectangles);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ax_interaction::xcode::get_xcode_editor_content;

    use super::super::SearchRule;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_rectangles() {
        let editor_pid = 25403 as i32;
        if let Ok(editor_content_option) = get_xcode_editor_content(editor_pid) {
            if let Some(editor_content) = editor_content_option {
                let search_str = "text ever since".to_string();
                let mut rule = SearchRule::new();
                rule.run(&editor_content, &search_str);

                if let Some(mut matches) = rule.rule_matches {
                    for single_match in matches.iter_mut() {
                        (*single_match).update_rectangles(editor_pid);
                    }

                    assert_eq!(matches.len(), 1);
                } else {
                    assert!(false);
                }
            } else {
                assert!(false, "Focused UI element is not a textarea");
            }
        } else {
            assert!(false, "Can not get editor content; presumable XCode is not running or focused UI element is not textarea");
        }

        // Observed "odd" behavior:
        // - If match includes last character of file, the bounds always stretch to the far end of textarea
        // - Word Wrap always draws the bounding around the whole text area (horizontally)
        // - When matching the last characters of a string that wraps around lines, the rect always extents to the maximum end text area's extents
        // - can not detect yet on which characters wordwrap appears

        // Figuring out word wrap:
        // - every line match rectangle should have the same height as the height a single character
        // - if the line match rectangle hight is greater than the height of a single character, then the text of this line is wrapped
        // - line match rectangle height devided by the height of a single character gives us the number of lines that the text is wrapped into
        // - First line gets a rectangle from the FIRST character rectangle of the match string to the RIGHT end of the text area
        // - Last line gets a rectangle from the LAST character rectangle of the match string to the LEFT end of the text area
        // - All lines inbetween get a rectangle stretching from the LEFT end of the text area to the RIGHT end of the text area
    }
}
