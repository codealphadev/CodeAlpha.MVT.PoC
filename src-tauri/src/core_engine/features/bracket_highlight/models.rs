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
    KnownElbow(f64),           // Includes wrapped line case
    EstimatedElbowOffset(f64), // Case: we're missing information
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
    pub lines: BracketHighlightLines,
    pub boxes: BracketHighlightBoxPair,
}

impl BracketHighlightResults {
    pub fn to_local(&self, global_origin: &LogicalPosition) -> Self {
        Self {
            boxes: {
                BracketHighlightBoxPair {
                    opening_bracket: self
                        .boxes
                        .opening_bracket
                        .map(|rect| rect.to_local(global_origin)),
                    closing_bracket: self
                        .boxes
                        .closing_bracket
                        .map(|rect| rect.to_local(global_origin)),
                }
            },
            lines: {
                BracketHighlightLines {
                    start: self.lines.start.map(|pos| pos.to_local(global_origin)),
                    end: self.lines.end.map(|pos| pos.to_local(global_origin)),
                    elbow: match self.lines.elbow {
                        Some(Elbow::KnownElbow(pos)) => {
                            Some(Elbow::KnownElbow(pos - global_origin.x))
                        }
                        Some(Elbow::EstimatedElbowOffset(offset)) => {
                            Some(Elbow::EstimatedElbowOffset(offset))
                        }
                        None => None,
                    },
                }
            },
        }
    }
}
