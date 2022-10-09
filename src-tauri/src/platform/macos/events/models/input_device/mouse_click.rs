use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utils::geometry::LogicalPosition;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub enum MouseButton {
    Left,
    Right,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub enum ClickType {
    Down,
    Up,
    Drag,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MouseClickMessage {
    pub button: MouseButton,
    pub click_type: ClickType,
    pub cursor_position: LogicalPosition,
}
