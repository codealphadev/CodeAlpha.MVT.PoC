#![allow(dead_code)]

use tauri::Manager;

use crate::window_controls::config::AppWindow;

use super::{close::close_window, open::open_window};

pub fn toggle_window(handle: &tauri::AppHandle, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        let res = app_window.is_visible();
        if let Ok(visible) = res {
            if visible {
                close_window(&handle, window_label);
            } else {
                open_window(&handle, window_label);
            }
        } else {
            println!("Err: {:?}", res);
        }
    } else {
        open_window(&handle, window_label);
    }
}
