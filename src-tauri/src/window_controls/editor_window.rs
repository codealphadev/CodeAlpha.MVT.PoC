#![allow(dead_code)]

use crate::ax_interaction::models::editor::{EditorWindowCreatedMessage, FocusedUIElement};

#[derive(Debug)]
enum HorizontalBoundary {
    Left,
    Right,
}

#[derive(Debug)]
enum VerticalBoundary {
    Top,
    Bottom,
}

#[derive(Debug)]
pub struct EditorWindow {
    pub id: uuid::Uuid,
    editor_name: String,
    pid: i32,
    pub focused_ui_element: Option<FocusedUIElement>,
    window_position: tauri::LogicalPosition<f64>,
    window_size: tauri::LogicalSize<f64>,
    textarea_position: Option<tauri::LogicalPosition<f64>>,
    textarea_size: Option<tauri::LogicalSize<f64>>,
    h_boundary: HorizontalBoundary,
    v_boundary: VerticalBoundary,
}

impl EditorWindow {
    pub fn new(created_msg: &EditorWindowCreatedMessage) -> Self {
        Self {
            id: created_msg.id,
            editor_name: created_msg.editor_name.clone(),
            pid: created_msg.pid,
            window_position: created_msg.window_position,
            window_size: created_msg.window_size,
            textarea_position: None,
            textarea_size: None,
            focused_ui_element: None,
            h_boundary: HorizontalBoundary::Right,
            v_boundary: VerticalBoundary::Bottom,
        }
    }

    pub fn update_window_dimensions(
        &mut self,
        position: tauri::LogicalPosition<f64>,
        size: tauri::LogicalSize<f64>,
    ) {
        self.window_position = position;
        self.window_size = size;
    }

    pub fn update_textarea_dimensions(
        &mut self,
        position: tauri::LogicalPosition<f64>,
        size: tauri::LogicalSize<f64>,
    ) {
        self.textarea_position = Some(position);
        self.textarea_size = Some(size);
    }

    pub fn update_focused_ui_element(
        &mut self,
        focused_ui_element: &FocusedUIElement,
        textarea_position: Option<tauri::LogicalPosition<f64>>,
        textarea_size: Option<tauri::LogicalSize<f64>>,
    ) {
        if let Some(position) = textarea_position {
            self.textarea_position = Some(position);
        }

        if let Some(size) = textarea_size {
            self.textarea_size = Some(size);
        }

        self.focused_ui_element = Some(focused_ui_element.clone());
    }
}
