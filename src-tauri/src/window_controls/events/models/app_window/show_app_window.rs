use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    utils::geometry::{LogicalFrame, LogicalPosition},
    window_controls::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct ShowAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
    pub editor_textarea: LogicalFrame,
    pub widget_position: Option<LogicalPosition>,
    pub monitor: LogicalFrame,
}
