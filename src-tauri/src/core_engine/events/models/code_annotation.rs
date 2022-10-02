use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{core_engine::EditorWindowUid, utils::geometry::LogicalFrame};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/node_annotation/")]
pub struct UpdateNodeAnnotationMessage {
    pub id: uuid::Uuid,
    pub window_uid: EditorWindowUid,
    pub annotation_icon: Option<LogicalFrame>,
    pub annotation_codeblock: Option<LogicalFrame>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/node_annotation/")]
pub struct RemoveNodeAnnotationMessage {
    pub id: uuid::Uuid,
    pub window_uid: EditorWindowUid,
}
