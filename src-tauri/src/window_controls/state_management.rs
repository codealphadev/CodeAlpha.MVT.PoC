use std::sync::{Arc, Mutex};

use tauri::{EventHandler, Manager};

use super::{create_window, AppWindow};

pub struct WindowStateManager {
    tauri_app_handle: tauri::AppHandle,

    // List of listeners; stored to be able to safely remove them
    listener_app_focus_status: Option<EventHandler>,
    listener_xcode_focus_status_change: Option<EventHandler>,

    last_known_editor_position: Arc<Mutex<Option<tauri::PhysicalPosition<i32>>>>,
    last_known_editor_size: Arc<Mutex<Option<tauri::PhysicalSize<i32>>>>,
    last_repositioned_widget_position: Arc<Mutex<Option<tauri::PhysicalPosition<i32>>>>,
    last_focused_app: Arc<Mutex<Option<String>>>,
}

impl WindowStateManager {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            tauri_app_handle: app_handle.clone(),
            listener_app_focus_status: None,
            listener_xcode_focus_status_change: None,
            last_known_editor_position: Arc::new(Mutex::new(None)),
            last_known_editor_size: Arc::new(Mutex::new(None)),
            last_repositioned_widget_position: Arc::new(Mutex::new(None)),
            last_focused_app: Arc::new(Mutex::new(None)),
        }
    }

    pub fn launch_startup_windows(&self) {
        let startup_window_list: [AppWindow; 1] = [AppWindow::Widget];

        for window_type in startup_window_list.iter() {
            let _ = create_window(&self.tauri_app_handle, *window_type);
        }
    }
}

impl Drop for WindowStateManager {
    fn drop(&mut self) {
        // Unregister listener for AppFocusStatus
        if let Some(listener_app_focus_status) = self.listener_app_focus_status.take() {
            self.tauri_app_handle.unlisten(listener_app_focus_status);
        }

        // Unregister listener for XCodeFocusStatusChange
        if let Some(listener_xcode_focus_status_change) =
            self.listener_xcode_focus_status_change.take()
        {
            self.tauri_app_handle
                .unlisten(listener_xcode_focus_status_change);
        }
    }
}
