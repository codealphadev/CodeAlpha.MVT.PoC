use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::EditorWindowUid,
    platform::macos::models::input_device::{ClickType, MouseButton},
    utils::geometry::LogicalPosition,
    window_controls::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct TrackingAreaClickedOutsideMessage {
    pub id: uuid::Uuid,
    pub window_uid: EditorWindowUid,
    pub app_window: AppWindow,
    pub mouse_position: LogicalPosition,
    pub button: MouseButton,
    pub click_type: ClickType,
}
