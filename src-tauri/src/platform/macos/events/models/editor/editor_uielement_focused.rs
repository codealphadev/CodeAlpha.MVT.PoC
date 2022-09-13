use serde::{Deserialize, Serialize};

use crate::platform::macos::{CodeDocumentFrameProperties, ViewportProperties};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FocusedUIElement {
    Textarea,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorUIElementFocusedMessage {
    pub window_uid: Option<usize>,
    pub pid: Option<i32>,
    pub focused_ui_element: FocusedUIElement,
    pub textarea_position: Option<tauri::LogicalPosition<f64>>,
    pub textarea_size: Option<tauri::LogicalSize<f64>>,
    pub viewport: Option<ViewportProperties>,
    pub code_document: Option<CodeDocumentFrameProperties>,
}
