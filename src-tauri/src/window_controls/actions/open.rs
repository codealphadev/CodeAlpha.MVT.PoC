use core::panic;

use tauri::{Error, Manager};

use crate::window_controls::{config::AppWindow, get_window_label};

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

    match window_label {
        AppWindow::Content => {
            let _ = special_open_for_content_window(handle);
        }
        _ => {
            if let Some(app_window) = handle.get_window(&get_window_label(window_label)) {
                let _ = app_window.show();
            } else {
                let _window = create_window(&handle, window_label);
            }
        }
    }
}

pub fn is_visible(handle: &tauri::AppHandle, window_label: AppWindow) -> bool {
    if window_label == AppWindow::None {
        return false;
    }

    if let Some(window) = handle.get_window(&get_window_label(window_label)) {
        if window.is_visible().unwrap() {
            return true;
        }
    }
    false
}

fn special_open_for_content_window(handle: &tauri::AppHandle) -> Result<(), Error> {
    if let Some(content_window) = handle.get_window(&get_window_label(AppWindow::Content)) {
        let _ = content_window.show();
    } else {
        // Create Window -> only when creating a new window the parent/child relationship needed for dragging is established.
        // We sacrifice other UX here, because window creation takes a noticable split second
        // Before we call "create", we need to get the updated position for the content window
        let content_window = create_window(&handle, AppWindow::Content)?;

        // Show Content window
        content_window.show()?;
    }

    Ok(())
}
