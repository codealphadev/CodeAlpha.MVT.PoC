use serde::{Deserialize, Serialize};

use crate::window_controls::config::AppWindow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppWindowMovedMessage {
    pub window: AppWindow,
    pub window_position: tauri::LogicalPosition<f64>,
}
