use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::{features::NodeExplanation, WindowUid},
    utils::geometry::LogicalFrame,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]
pub struct NodeExplanationFetchedMessage {
    pub explanation: NodeExplanation,
    pub name: String,
    pub window_uid: WindowUid,
    pub annotation_frame: Option<LogicalFrame>,
}
