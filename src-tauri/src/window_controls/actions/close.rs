use tauri::Manager;

use crate::window_controls::config::AppWindow;

pub fn close_window(handle: &tauri::AppHandle, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    if window_label == AppWindow::Content {
        panic!("Use open_window method of ContentWindow instead");
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        let _ = app_window.hide();
    }
}
