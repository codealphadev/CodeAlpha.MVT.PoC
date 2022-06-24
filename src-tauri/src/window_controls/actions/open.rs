use core::panic;

use tauri::Manager;

use crate::window_controls::config::AppWindow;

use super::create::create_window;

/// If the window is not visible, show it (or create it first if it doesn't exist).
///
/// Arguments:
///
/// * `handle`: The handle to the tauri app.
/// * `window_label`: This is the label of the window you want to open.
///
/// Returns:
///
/// A boolean value indicating if the showing the window was successful.
pub fn open_window(handle: &tauri::AppHandle, window_label: AppWindow) -> bool {
    if window_label == AppWindow::None {
        return false;
    }

    if window_label == AppWindow::Content {
        panic!("Use open_window method of ContentWindow instead");
    }

    if is_visible(&handle, window_label) {
        return false;
    }

    if let Some(app_window) = handle.get_window(&window_label.to_string()) {
        let _ = app_window.show();
        true
    } else {
        if create_window(&handle, window_label).is_ok() {
            true
        } else {
            false
        }
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
