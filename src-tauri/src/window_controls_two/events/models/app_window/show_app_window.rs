use serde::{Deserialize, Serialize};

use crate::{
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls_two::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShowAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
    pub editor_textarea_position: LogicalPosition,
    pub editor_textarea_size: LogicalSize,
    pub widget_position: Option<LogicalPosition>,
}
