use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::EditorWindowUid, utils::geometry::LogicalPosition,
    window_controls::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct TrackingAreaEnteredMessage {
    pub id: uuid::Uuid,
    pub window_uid: EditorWindowUid,
    pub app_window: AppWindow,
    pub mouse_position: LogicalPosition,
}
