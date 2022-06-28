use super::{RuleMatch, RuleName, RuleResults, SearchRule, SwiftLinterRule};

pub enum RuleType {
    SearchRule(SearchRule),
    SwiftLinter(SwiftLinterRule),
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
            RuleType::SearchRule(search_rule) => search_rule.rule_type(),
            RuleType::SwiftLinter(swift_linter_rule) => swift_linter_rule.rule_type(),
        }
    }

    fn rule_matches(&self) -> Option<&Vec<RuleMatch>> {
        match self {
            RuleType::SearchRule(search_rule) => search_rule.rule_matches(),
            RuleType::SwiftLinter(swift_linter_rule) => swift_linter_rule.rule_matches(),
        }
    }

    fn rule_results(&self) -> Option<RuleResults> {
        match self {
            RuleType::SearchRule(search_rule) => search_rule.rule_results(),
            RuleType::SwiftLinter(swift_linter_rule) => swift_linter_rule.rule_results(),
        }
    }

    fn run(&mut self) -> Option<RuleResults> {
        match self {
            RuleType::SearchRule(search_rule) => search_rule.run(),
            RuleType::SwiftLinter(swift_linter_rule) => swift_linter_rule.run(),
        }
    }

    fn compute_rule_match_rectangles(&mut self, editor_app_pid: i32) -> Option<RuleResults> {
        match self {
            RuleType::SearchRule(search_rule) => {
                search_rule.compute_rule_match_rectangles(editor_app_pid)
            }
            RuleType::SwiftLinter(swift_linter_rule) => {
                swift_linter_rule.compute_rule_match_rectangles(editor_app_pid)
            }
        }
    }
}
