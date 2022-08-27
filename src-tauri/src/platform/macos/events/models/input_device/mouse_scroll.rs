use serde::{Deserialize, Serialize};

use crate::utils::geometry::LogicalPosition;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MouseScrollMessage {
    pub scroll_delta_x: i64,
    pub scroll_delta_y: i64,
    pub cursor_position: LogicalPosition,
}
