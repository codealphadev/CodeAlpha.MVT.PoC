#![allow(dead_code)]

enum HorizontalBoundary {
    Left,
    Right,
}

enum VerticalBoundary {
    Top,
    Bottom,
}

pub struct EditorWindow {
    id: uuid::Uuid,
    window_position: tauri::LogicalPosition<i32>,
    window_size: tauri::LogicalSize<i32>,
    textarea_position: tauri::LogicalPosition<i32>,
    textarea_size: tauri::LogicalSize<i32>,
    h_boundary: HorizontalBoundary,
    v_boundary: VerticalBoundary,
    handle: tauri::AppHandle,
}

impl EditorWindow {}
