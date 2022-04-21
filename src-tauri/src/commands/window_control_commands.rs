use tauri::Manager;

use crate::window_controls::{
    actions::{close_window, open_window, resize_window, toggle_window},
    get_window_label, AppWindow,
};

#[tauri::command]
pub fn cmd_is_window_visible(handle: tauri::AppHandle, window_type: AppWindow) -> bool {
    if window_type == AppWindow::None {
        return false;
    }

    let app_window = handle.get_window(&get_window_label(window_type));

    if let Some(app_window) = app_window {
        if let Ok(visible) = app_window.is_visible() {
            if visible {
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }
}

#[tauri::command]
pub fn cmd_open_window(handle: tauri::AppHandle, window_label: AppWindow) {
    open_window(&handle, window_label);
}

#[tauri::command]
pub fn cmd_close_window(handle: tauri::AppHandle, window_label: AppWindow) {
    close_window(&handle, window_label);
}

#[tauri::command]
pub fn cmd_toggle_window(handle: tauri::AppHandle, window_label: AppWindow) {
    toggle_window(&handle, window_label);
}

#[tauri::command]
pub fn cmd_resize_window(
    handle: tauri::AppHandle,
    window_label: AppWindow,
    size_x: u32,
    size_y: u32,
) {
    let _ = resize_window(
        &handle,
        window_label,
        &tauri::LogicalSize {
            width: size_x as f64,
            height: size_y as f64,
        },
    );
}
