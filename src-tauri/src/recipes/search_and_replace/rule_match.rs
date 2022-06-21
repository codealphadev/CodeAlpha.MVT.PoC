use accessibility::{AXAttribute, AXValue};
use core_foundation::{base::CFRange, number::CFNumber};
use core_graphics_types::geometry::CGRect;
use tauri::{LogicalPosition, LogicalSize};

use crate::ax_interaction::focused_uielement_of_app;

#[derive(Debug)]
pub struct Rectangle {
    pub origin: LogicalPosition<f64>,
    pub size: LogicalSize<f64>,
}

#[derive(Debug)]
pub struct MatchRange {
    pub index: usize,
    pub length: usize,
}

#[derive(Debug)]
pub struct RuleMatch {
    pub range: MatchRange,
    pub matched: String,
    pub rectangles: Vec<Rectangle>,
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

            let mut current_match_index = self.range.index;
            while let Ok(line_number) = editor_textarea_ui_element.parameterized_attribute(
                &AXAttribute::line_for_index(),
                &CFNumber::from(current_match_index as i64),
            ) {
                // Get line length
                if let Ok(line_range_as_axval) = editor_textarea_ui_element
                    .parameterized_attribute(&AXAttribute::range_for_line(), &line_number)
                {
                    // Convert AXValue into valid CFRange
                    let line_range = dbg!(line_range_as_axval).get_value::<CFRange>().unwrap();
                    let range_index = line_range.location as usize;
                    let range_length = line_range.length as usize;

                    let line_match_range = MatchRange {
                        index: current_match_index,
                        length: std::cmp::min(
                            range_length - (current_match_index - range_index),
                            self.range.length - (current_match_index - self.range.index),
                        ),
                    };

                    current_match_index = current_match_index + line_match_range.length;
                    line_match_ranges.push(line_match_range);

                    if current_match_index >= self.range.index + self.range.length {
                        break;
                    }
                }
            }

            dbg!(&line_match_ranges);

            let mut line_match_range_rectangles: Vec<Rectangle> = Vec::new();
            for line_match_range in line_match_ranges {
                // Get line rectangles
                let axval_from_cfrange = AXValue::from_CFRange(CFRange {
                    location: line_match_range.index as isize,
                    length: line_match_range.length as isize,
                })
                .unwrap();

                if let Ok(line_match_rectangle_as_axval) = editor_textarea_ui_element
                    .parameterized_attribute(&AXAttribute::bounds_for_range(), &axval_from_cfrange)
                {
                    // Convert AXValue into valid CGRect
                    let line_rectangle =
                        line_match_rectangle_as_axval.get_value::<CGRect>().unwrap();

                    let line_match_rectangle = Rectangle {
                        origin: tauri::LogicalPosition {
                            x: line_rectangle.origin.x as f64,
                            y: line_rectangle.origin.y as f64,
                        },
                        size: LogicalSize {
                            width: line_rectangle.size.width as f64,
                            height: line_rectangle.size.height as f64,
                        },
                    };

                    line_match_range_rectangles.push(line_match_rectangle);
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
        let editor_pid = 12538 as i32;
        if let Ok(editor_content_option) = get_xcode_editor_content(editor_pid) {
            if let Some(editor_content) = editor_content_option {
                let search_str = "text ever since ".to_string();
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
            }
        }

        // Observed "odd" behavior:
        // - Word Wrap always draws the bounding around the whole text area (horizontally)
        // - When matching the last characters a string that wraps around lines, the rect always extents to the maximum end text area's extents
        // - can not detenct yet on which characters wordwrap appears
    }
}
