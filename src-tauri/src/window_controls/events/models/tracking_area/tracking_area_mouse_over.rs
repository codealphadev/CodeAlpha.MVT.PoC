use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::EditorWindowUid, utils::geometry::LogicalPosition,
    window_controls::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct TrackingAreaMouseOverMessage {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
    pub app_window: AppWindow,
    pub mouse_position: LogicalPosition,

    /// The duration in millis from when the tracking area was entered to now.
    pub duration_ms: u64,
}
