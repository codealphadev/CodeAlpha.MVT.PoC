use crate::{
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls_two::config::AppWindow,
};

pub struct UpdateDimensionsAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
    pub editor_textarea_position: LogicalPosition,
    pub editor_textarea_size: LogicalSize,
    pub widget_position: Option<LogicalPosition>,
}
