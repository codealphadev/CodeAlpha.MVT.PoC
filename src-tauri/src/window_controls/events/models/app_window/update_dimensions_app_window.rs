use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct UpdateDimensionsAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
    pub editor_textarea_position: LogicalPosition,
    pub editor_textarea_size: LogicalSize,
    pub widget_position: Option<LogicalPosition>,
}
