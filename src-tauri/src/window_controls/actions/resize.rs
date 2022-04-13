use tauri::Manager;

use crate::window_controls::config::AppWindow;

pub fn resize_window(
    handle: &tauri::AppHandle,
    window_label: AppWindow,
    size: &tauri::LogicalSize<f64>,
) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        let _ = app_window.set_size(tauri::Size::Logical(*size));
    }
}
