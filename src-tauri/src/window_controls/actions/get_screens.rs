#![allow(dead_code)]

use tauri::{Manager, Monitor};

use crate::window_controls::config::AppWindow;

/// Returns the `tauri::Monitor` object which the window of `window_label` is currently
/// positioned on. If a `window_label` of a non-existing window is provided the result will
/// be `None`.
pub fn current_monitor_of_window(
    app_handle: &tauri::AppHandle,
    window_label: AppWindow,
) -> Option<Monitor> {
    if window_label == AppWindow::None {
        return None;
    }

    if let Some(app_window) = app_handle.get_window(&window_label.to_string()) {
        if let Ok(monitor) = app_window.current_monitor() {
            monitor
        } else {
            None
        }
    } else {
        None
    }
}

/// Returns all monitors available to tauri which is usually all screens the user has in front of her.
pub fn available_monitors(
    app_handle: &tauri::AppHandle,
    created_window: AppWindow,
) -> Vec<Monitor> {
    if created_window == AppWindow::None {
        return Vec::new();
    }

    if let Some(app_window) = app_handle.get_window(&created_window.to_string()) {
        if let Ok(monitor) = app_window.available_monitors() {
            monitor
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

pub fn primary_monitor(app_handle: &tauri::AppHandle, window_label: AppWindow) -> Option<Monitor> {
    if window_label == AppWindow::None {
        return None;
    }

    if let Some(app_window) = app_handle.get_window(&window_label.to_string()) {
        if let Ok(monitor) = app_window.primary_monitor() {
            monitor
        } else {
            None
        }
    } else {
        None
    }
}
