use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{core_engine::WindowUid, utils::geometry::LogicalFrame};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]
pub struct UpdateNodeAnnotationMessage {
    pub id: uuid::Uuid,
    pub window_uid: WindowUid,
    pub annotation_icon: Option<LogicalFrame>,
    pub annotation_codeblock: Option<LogicalFrame>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]
pub struct RemoveNodeAnnotationMessage {
    pub id: uuid::Uuid,
    pub window_uid: WindowUid,
}
