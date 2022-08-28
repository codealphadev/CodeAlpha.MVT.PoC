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
pub struct BracketHighlightBoxPair {
    pub first: Option<LogicalFrame>,
    pub last: Option<LogicalFrame>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]

pub enum BracketHighlightElbow {
    LeftMost,
    ElbowPoint(LogicalPosition),
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]

pub struct BracketHighlightLines {
    pub first: Option<LogicalFrame>,
    pub last: Option<LogicalFrame>,
    pub elbow: Option<BracketHighlightElbow>,
    pub bottom_line_top: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/bracket_highlighting/")]
pub struct BracketHighlightResults {
    lines: BracketHighlightLines,
    boxes: BracketHighlightBoxPair,
}
