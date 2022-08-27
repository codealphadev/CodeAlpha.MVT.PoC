use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{swift_linter::LintLevel, RuleMatch, SwiftLinterRule};

pub enum RuleType {
    _SwiftLinter(SwiftLinterRule),
}

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

pub trait RuleBase {
    fn rule_type(&self) -> RuleName;
    fn rule_matches(&self) -> Option<&Vec<RuleMatch>>;
    fn rule_results(&self) -> Option<RuleResults>;
    fn run(&mut self) -> Option<RuleResults>;
    fn compute_rule_match_rectangles(&mut self, editor_app_pid: i32) -> Option<RuleResults>;
}

impl RuleBase for RuleType {
    fn rule_type(&self) -> RuleName {
        match self {
            RuleType::_SwiftLinter(rule) => rule.rule_type(),
        }
    }

    fn rule_matches(&self) -> Option<&Vec<RuleMatch>> {
        match self {
            RuleType::_SwiftLinter(rule) => rule.rule_matches(),
        }
    }

    fn rule_results(&self) -> Option<RuleResults> {
        match self {
            RuleType::_SwiftLinter(rule) => rule.rule_results(),
        }
    }

    fn run(&mut self) -> Option<RuleResults> {
        match self {
            RuleType::_SwiftLinter(rule) => rule.run(),
        }
    }

    fn compute_rule_match_rectangles(&mut self, editor_app_pid: i32) -> Option<RuleResults> {
        match self {
            RuleType::_SwiftLinter(rule) => rule.compute_rule_match_rectangles(editor_app_pid),
        }
    }
}
