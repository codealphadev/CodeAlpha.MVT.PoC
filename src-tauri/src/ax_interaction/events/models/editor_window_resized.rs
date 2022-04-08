use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorWindowResizedMessage {
    pub id: uuid::Uuid,
    pub window_position: tauri::LogicalPosition<f64>,
    pub window_size: tauri::LogicalSize<f64>,
    pub editor_position: Option<tauri::LogicalPosition<f64>>,
    pub editor_size: Option<tauri::LogicalSize<f64>>,
}
