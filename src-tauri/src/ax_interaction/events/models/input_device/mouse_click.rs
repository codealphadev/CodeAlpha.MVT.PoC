use serde::{Deserialize, Serialize};

use crate::utils::geometry::LogicalPosition;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ClickType {
    Down,
    Up,
    Drag,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MouseClickMessage {
    pub button: MouseButton,
    pub click_type: ClickType,
    pub cursor_position: LogicalPosition,
}
