use tauri::Manager;

use crate::window_controls::{config::AppWindow, get_window_label};

use super::{close::close_window, open::open_window};

pub fn toggle_window(handle: &tauri::AppHandle, window_type: AppWindow) {
    if window_type == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&get_window_label(window_type));

    if let Some(app_window) = app_window {
        if app_window.is_visible().unwrap() {
            close_window(&handle, window_type);
        } else {
            open_window(&handle, window_type);
        }
    } else {
        open_window(&handle, window_type);
    }
}
