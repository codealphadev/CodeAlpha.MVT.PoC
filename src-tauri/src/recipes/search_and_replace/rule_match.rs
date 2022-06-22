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
        // 1. Get Editor TextArea UI Element
        if let Ok(editor_textarea_ui_element) = focused_uielement_of_app(editor_app_pid) {
            let mut line_match_ranges: Vec<MatchRange> = Vec::new();

            // 2. Break up match range into individual matches that only span one line in the editor
            let mut current_match_index = self.range.index;
            while let Some(line_number) = get_line_number_for_range_index(
                current_match_index as i64,
                &editor_textarea_ui_element,
            ) {
                if let Some(current_line_range) =
                    get_char_range_of_line(line_number, &editor_textarea_ui_element)
                {
                    let line_match_range = MatchRange {
                        index: current_match_index,
                        length: std::cmp::min(
                            current_line_range.length
                                - (current_match_index - current_line_range.index),
                            self.range.length - (current_match_index - self.range.index),
                        ),
                    };

                    // Add +1 because current_line_range got its last char removed because it is always a line break character '\n'.
                    // If we would not remove it, the calculated rectangles would stretch the the line to the end of the line.
                    current_match_index = current_match_index + line_match_range.length + 1;
                    line_match_ranges.push(line_match_range);

                    if current_match_index >= self.range.index + self.range.length {
                        break;
                    }
                }
            }

            // 3. Calculate rectangles for each line match range; checking if they are wrapped, potentially adding multiple rectangles
            let mut line_match_range_rectangles: Vec<MatchRectangle> = Vec::new();
            for line_match_range in line_match_ranges {
                // Check if line_match_range actually wraps into multiple lines
                // due to activated 'wrap lines' in XCode (default is on)
                if let Some((range_is_wrapping, wrapped_line_number)) =
                    is_text_of_line_wrapped(&line_match_range, &editor_textarea_ui_element)
                {
                    if !range_is_wrapping {
                        if let Some(line_match_rect) =
                            get_bounds_of_MatchRange(&line_match_range, &editor_textarea_ui_element)
                        {
                            line_match_range_rectangles.push(line_match_rect);
                        }
                    } else {
                        line_match_range_rectangles.extend(calc_match_rects_for_wrapped_range(
                            wrapped_line_number,
                            &line_match_range,
                            &editor_textarea_ui_element,
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ax_interaction::xcode::get_xcode_editor_content;

    use super::super::SearchRule;

    #[test]
    fn test_get_rectangles() {
        let editor_pid = 12538 as i32;
        if let Ok(editor_content_option) = get_xcode_editor_content(editor_pid) {
            if let Some(editor_content) = editor_content_option {
                let search_str =
                    "// swift-tools-version:4.0\n\nimport PackageDescription\n\nlet".to_string();
                let mut rule = SearchRule::new();
                rule.run(&editor_content, &search_str);
                rule.compute_match_boundaries(editor_pid);
            } else {
                assert!(false, "Focused UI element is not a textarea");
            }
        } else {
            assert!(false, "Can not get editor content; presumable XCode is not running or focused UI element is not textarea");
        }
    }
}
