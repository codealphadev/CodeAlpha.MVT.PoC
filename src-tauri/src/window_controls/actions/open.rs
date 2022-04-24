use core::panic;

use tauri::Manager;

use crate::window_controls::config::AppWindow;

use super::create::create_window;

pub fn open_window(handle: &tauri::AppHandle, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    if window_label == AppWindow::Content {
        panic!("Use open_window method of ContentWindow instead");
    }

    if is_visible(&handle, window_label) {
        return;
    }

    if let Some(app_window) = handle.get_window(&window_label.to_string()) {
        let _ = app_window.show();
    } else {
        let _window = create_window(&handle, window_label);
    }
}

pub fn is_visible(handle: &tauri::AppHandle, window_label: AppWindow) -> bool {
    if window_label == AppWindow::None {
        return false;
    }

    if let Some(window) = handle.get_window(&window_label.to_string()) {
        if window.is_visible().unwrap() {
            return true;
        }
    }
    false
}
