use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FocusedUIElement {
    Textarea,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorUIElementFocusedMessage {
    pub window_id: uuid::Uuid,
    pub ui_elem_hash: usize,
    pub pid: i32,
    pub focused_ui_element: FocusedUIElement,
    pub textarea_position: Option<tauri::LogicalPosition<f64>>,
    pub textarea_size: Option<tauri::LogicalSize<f64>>,
}
