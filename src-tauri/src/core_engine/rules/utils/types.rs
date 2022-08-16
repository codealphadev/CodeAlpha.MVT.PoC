use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::rules::swift_linter::LintLevel,
    core_engine::rules::RuleMatch,
    utils::{
        geometry::{LogicalPosition, LogicalSize},
        grapheme::grapheme_slice_string,
    },
};

use super::text_types::TextRange;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/")]
pub enum RuleName {
    BracketHighlight,
    SearchAndReplace,
    SwiftLinter,
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/")]
pub struct RuleResults {
    pub rule: RuleName,
    pub results: Vec<RuleMatch>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct MatchRectangle {
    pub origin: LogicalPosition,
    pub size: LogicalSize,
}

impl MatchRectangle {
    pub fn contains_point(&self, mouse_x: f64, mouse_y: f64) -> bool {
        // Check if mouse_x and mouse_y are within the bounds of the rectangle.
        let x_in_bounds = mouse_x >= self.origin.x && mouse_x <= self.origin.x + self.size.width;
        let y_in_bounds = mouse_y >= self.origin.y && mouse_y <= self.origin.y + self.size.height;
        x_in_bounds && y_in_bounds
    }
}

pub type LineMatch = (MatchRange, Vec<MatchRectangle>);

#[cfg(test)]
mod tests_MatchRectangle {

    use super::MatchRectangle;
    use crate::utils::geometry::{LogicalPosition, LogicalSize};

    #[test]
    fn test_contains_point() {
        let rectangle = MatchRectangle {
            origin: LogicalPosition { x: 0.0, y: 0.0 },
            size: LogicalSize {
                width: 100.0,
                height: 100.0,
            },
        };

        assert!(rectangle.contains_point(50., 50.));
        assert!(rectangle.contains_point(0., 0.));
        assert!(rectangle.contains_point(100., 100.));
        assert!(!rectangle.contains_point(101., 100.));
        assert!(!rectangle.contains_point(100., 101.));
        assert!(!rectangle.contains_point(150., 150.));
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct MatchRange {
    pub string: String,
    pub range: TextRange,
}

impl MatchRange {
    pub fn from_text_and_range(text: &String, range: TextRange) -> Option<Self> {
        Some(Self {
            string: grapheme_slice_string(text, range.index, range.length)?,
            range,
        })
    }
}

#[cfg(test)]
mod tests_MatchRange {
    use super::MatchRange;
    use crate::core_engine::rules::TextRange;
    use pretty_assertions::assert_eq;

    fn test_from_text_and_range(input_text: &str, range: TextRange, expected_string: Option<&str>) {
        let match_range = MatchRange::from_text_and_range(&input_text.to_string(), range);
        if let Some(match_range) = match_range {
            assert_eq!(match_range.string, expected_string.unwrap());
            assert_eq!(match_range.range, range);
        } else {
            assert!(expected_string.is_none());
        }
    }

    #[test]
    fn from_text_and_range() {
        test_from_text_and_range(
            "0123456789",
            TextRange {
                index: 2,
                length: 5,
            },
            Some("23456"),
        )
    }

    #[test]
    fn from_text_and_range_out_of_range() {
        test_from_text_and_range(
            "0123456789",
            TextRange {
                index: 10,
                length: 5,
            },
            None,
        )
    }

    #[test]
    fn from_text_and_range_unicode() {
        test_from_text_and_range(
            "Hey © unicode chars",
            TextRange {
                index: 2,
                length: 5,
            },
            Some("y © u"),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub enum RuleMatchCategory {
    Error,
    Warning,
    BracketHighlightLineFirst,
    BracketHighlightLineLast,
    None,
}

impl RuleMatchCategory {
    pub fn from_lint_level(lint_level: LintLevel) -> RuleMatchCategory {
        match lint_level {
            LintLevel::Error => RuleMatchCategory::Error,
            LintLevel::Warning => RuleMatchCategory::Warning,
        }
    }
}
