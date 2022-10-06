use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{core_engine::EditorWindowUid, utils::geometry::LogicalFrame};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct NodeExplanationFetchedMessage {
    pub editor_window_uid: EditorWindowUid,
    pub annotation_frame: Option<LogicalFrame>,
}
