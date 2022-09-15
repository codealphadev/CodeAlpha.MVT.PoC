use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::features::NodeExplanation;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/node_explanation/")]
pub struct UpdateNodeExplanationMessage {
    pub explanation: NodeExplanation,
    pub name: Option<String>,
    pub complexity: Option<isize>,
}
