use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::rules::swift_linter::LintLevel,
    core_engine::rules::RuleMatch,
    utils::geometry::{LogicalPosition, LogicalSize},
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub enum RuleMatchCategory {
    Error,
    Warning,
    BracketHighlightLineFirst,
    BracketHighlightLineLast,
    BracketHighlightTouchFirst,
    BracketHighlightTouchLast,
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
