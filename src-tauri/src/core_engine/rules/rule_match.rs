use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ax_interaction::get_textarea_uielement;

use super::{calc_rectangles_and_line_matches, LineMatch, RuleMatchCategory, RuleName};

use super::utils::types::{MatchRange, MatchRectangle};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/")]
pub struct RuleMatchProps {
    pub identifier: String,
    pub description: String,
    pub category: RuleMatchCategory,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/")]
pub struct RuleMatch {
    id: uuid::Uuid,
    match_range: MatchRange,
    line_matches: Vec<LineMatch>,
    rectangles: Vec<MatchRectangle>,
    rule_name: RuleName,
    match_properties: RuleMatchProps,
}

impl RuleMatch {
    pub fn new(
        rule_name: RuleName,
        match_range: MatchRange,
        match_properties: RuleMatchProps,
    ) -> Self {
        Self {
            match_range,
            rectangles: Vec::new(),
            line_matches: Vec::new(),
            id: uuid::Uuid::new_v4(),
            rule_name,
            match_properties,
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
        if let Some(textarea_ui_element) = get_textarea_uielement(editor_app_pid) {
            let (rule_match_rectangles, line_matches) =
                calc_rectangles_and_line_matches(&self.match_range, &textarea_ui_element);

            self.rectangles = rule_match_rectangles;
            self.line_matches = line_matches;
        }
    }
}

#[cfg(test)]
mod tests {}
