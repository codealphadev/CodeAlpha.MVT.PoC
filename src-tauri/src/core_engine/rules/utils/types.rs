use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::rules::RuleMatch,
    utils::geometry::{LogicalPosition, LogicalSize},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/")]
pub enum RuleName {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct CharRange {
    pub index: usize,
    pub length: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct MatchRange {
    pub string: String,
    pub range: CharRange,
}
