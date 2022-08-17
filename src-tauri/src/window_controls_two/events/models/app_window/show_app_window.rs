use serde::{Deserialize, Serialize};

use crate::{
    utils::geometry::{LogicalFrame, LogicalPosition},
    window_controls_two::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShowAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
    pub editor_textarea: LogicalFrame,
    pub widget_position: Option<LogicalPosition>,
    pub monitor: LogicalFrame,
}
