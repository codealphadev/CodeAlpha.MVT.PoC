use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utils::geometry::{LogicalFrame, LogicalPosition};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightBracket {
    pub rectangle: LogicalFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub enum Elbow {
    KnownElbow(LogicalPosition),     // Includes wrapped line case
    EstimatedElbow(LogicalPosition), // Case: we're missing information
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightLines {
    pub start: Option<LogicalPosition>,
    pub end: Option<LogicalPosition>,
    pub elbow: Option<Elbow>, // None doesn't mean recompute; just means there's no elbow
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightBoxPair {
    pub opening_bracket: Option<LogicalFrame>,
    pub closing_bracket: Option<LogicalFrame>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightResults {
    lines: BracketHighlightLines,
    boxes: BracketHighlightBoxPair,
}
