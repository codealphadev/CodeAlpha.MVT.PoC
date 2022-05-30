use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EditorWindowCreatedMessage {
    pub id: uuid::Uuid,
    pub ui_elem_hash: usize,
    pub editor_name: String,
    pub pid: i32,
    pub window_position: tauri::LogicalPosition<f64>,
    pub window_size: tauri::LogicalSize<f64>,
}
