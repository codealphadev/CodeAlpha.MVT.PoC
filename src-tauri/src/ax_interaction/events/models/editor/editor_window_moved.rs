use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EditorWindowMovedMessage {
    pub id: uuid::Uuid,
    pub uielement_hash: usize,
    pub window_position: tauri::LogicalPosition<f64>,
    pub window_size: tauri::LogicalSize<f64>,
}
