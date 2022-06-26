use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ax_interaction::get_textarea_uielement;

use super::utils::ax_utils::{
    calc_match_rects_for_wrapped_range, get_bounds_of_CharRange, get_char_range_of_line,
    get_line_number_for_range_index, is_text_of_line_wrapped,
};

use super::utils::types::{CharRange, MatchRange, MatchRectangle};

type LineMatch = (MatchRange, Vec<MatchRectangle>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/")]
pub struct RuleMatch {
    id: uuid::Uuid,
    match_range: MatchRange,
    line_matches: Vec<LineMatch>,
    rectangles: Vec<MatchRectangle>,
}

impl RuleMatch {
    pub fn new(match_range: MatchRange) -> Self {
        Self {
            match_range,
            rectangles: Vec::new(),
            line_matches: Vec::new(),
            id: uuid::Uuid::new_v4(),
        }
    }

    #[allow(unused)]
    pub fn match_range(&self) -> &MatchRange {
        &self.match_range
    }

    #[allow(unused)]
    pub fn line_matches(&self) -> &Vec<LineMatch> {
        &self.line_matches
    }

    #[allow(unused)]
    pub fn rectangles(&self) -> &Vec<MatchRectangle> {
        &self.rectangles
    }

    pub fn update_rectangles(&mut self, editor_app_pid: i32) {
        // 1. Get Editor TextArea UI Element
        if let Some(editor_textarea_ui_element) = get_textarea_uielement(editor_app_pid) {
            let mut line_match_ranges: Vec<MatchRange> = Vec::new();

            // 2. Break up match range into individual matches that only span one line in the editor
            let mut current_match_index = self.match_range.range.index;
            while let Some(line_number) = get_line_number_for_range_index(
                current_match_index as i64,
                &editor_textarea_ui_element,
            ) {
                if let Some(current_line_range) =
                    get_char_range_of_line(line_number, &editor_textarea_ui_element)
                {
                    let matched_char_range = CharRange {
                        index: current_match_index,
                        length: std::cmp::min(
                            current_line_range.length
                                - (current_match_index - current_line_range.index),
                            self.match_range.range.length
                                - (current_match_index - self.match_range.range.index),
                        ),
                    };

                    let mut substr: String = String::new();
                    let mut matched_str_char_iter = self.match_range.string.char_indices();
                    for (i, c) in matched_str_char_iter.by_ref() {
                        if i >= matched_char_range.index - self.match_range.range.index
                            && i < (matched_char_range.index - self.match_range.range.index)
                                + matched_char_range.length
                        {
                            substr.push(c);
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

                    if current_match_index
                        >= self.match_range.range.index + self.match_range.range.length
                    {
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
                    is_text_of_line_wrapped(&line_match_range.range, &editor_textarea_ui_element)
                {
                    if !range_is_wrapping {
                        if let Some(line_match_rect) = get_bounds_of_CharRange(
                            &line_match_range.range,
                            &editor_textarea_ui_element,
                        ) {
                            rule_match_rectangles.push(line_match_rect.clone());
                            line_matches.push((line_match_range, vec![line_match_rect]));
                        }
                    } else {
                        let line_match_rectangles = calc_match_rects_for_wrapped_range(
                            wrapped_line_number,
                            &line_match_range.range,
                            &editor_textarea_ui_element,
                        );

                        rule_match_rectangles.extend(calc_match_rects_for_wrapped_range(
                            wrapped_line_number,
                            &line_match_range.range,
                            &editor_textarea_ui_element,
                        ));

                        line_matches.push((line_match_range, line_match_rectangles));
                    }
                }
            }

            self.rectangles = rule_match_rectangles;
            self.line_matches = line_matches;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ax_interaction::xcode::get_xcode_editor_content,
        core_engine::rules::search_and_replace::SearchRule,
    };

    #[test]
    #[ignore]
    fn test_get_rectangles() {
        let editor_pid = 12538 as i32;
        if let Ok(editor_content_option) = get_xcode_editor_content(editor_pid) {
            if let Some(editor_content) = editor_content_option {
                let search_str = "]\n)text ever since".to_string();
                let mut rule = SearchRule::new();
                rule.run(Some(editor_content), Some(search_str));
                rule.compute_match_boundaries(editor_pid);

                dbg!(rule.rule_matches());
            } else {
                assert!(false, "Focused UI element is not a textarea");
            }
        } else {
            assert!(false, "Can not get editor content; presumable XCode is not running or focused UI element is not textarea");
        }
    }
}
