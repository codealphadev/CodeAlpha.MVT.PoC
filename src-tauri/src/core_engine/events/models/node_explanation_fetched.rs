use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{core_engine::WindowUid, utils::geometry::LogicalFrame};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct NodeExplanationFetchedMessage {
    pub window_uid: WindowUid,
    pub annotation_frame: Option<LogicalFrame>,
}
