use tauri::{Error, Manager};

use crate::window_controls::{config::AppWindow, get_window_label};

use super::create::create_window;

pub fn open_window(handle: &tauri::AppHandle, window_type: AppWindow) {
    if window_type == AppWindow::None {
        return;
    }

    if let Some(window) = handle.get_window(&get_window_label(window_type)) {
        if window.is_visible().unwrap() {
            return;
        }
    }

    match window_type {
        AppWindow::Content => {
            let _ = special_open_for_content_window(handle);
        }
        _ => {
            if let Some(app_window) = handle.get_window(&get_window_label(window_type)) {
                let _ = app_window.show();
            } else {
                let _window = create_window(&handle, window_type);
            }
        }
    }
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
