use tauri::Manager;

use crate::window_controls::{config::AppWindow, get_window_label};

pub fn close_window(handle: tauri::AppHandle, window_type: AppWindow) {
    if window_type == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&get_window_label(window_type));

    if let Some(app_window) = app_window {
        match window_type {
            AppWindow::Content => special_close_for_content_window(&app_window),
            _ => {
                let _ = app_window.hide();
            }
        }
    }
}

fn special_close_for_content_window(content_window: &tauri::Window) {
    let _ = content_window.close();
}
