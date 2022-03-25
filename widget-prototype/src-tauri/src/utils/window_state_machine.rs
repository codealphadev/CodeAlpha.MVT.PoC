use super::window_controls::{cmd_close_window, cmd_open_window, AppWindow};
use crate::{
    utils::xcode_twin::XCodeTwin,
    websocket::{models, types},
};
use std::sync::{Arc, Mutex};
use tauri::{EventHandler, Manager};

// Features:
// [x] Which windows to load at startup
// [ ] Listening to movement of widget window --> updating position accordingly
//   [ ] Move logic from TS into Rust
//   [ ] Detect "GhostClicks in Rust instead of in Frontend"
// [x] Listening to XCode Twin messages and update window visibility accordingly

pub struct WindowStateMachine {
    tauri_app_handle: tauri::AppHandle,
    listener_app_focus_status: Option<EventHandler>,
    listener_xcode_focus_status_change: Option<EventHandler>,

    preserve_content_visibility_was_visible: Arc<Mutex<bool>>,
}

impl WindowStateMachine {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        // Init preserve_content_visibility_was_visible
        let mut preserve_content_visibility_was_visible = Arc::new(Mutex::new(false));
        let content_window = app_handle.get_window(&AppWindow::Content.to_string());
        if content_window.is_some() {
            preserve_content_visibility_was_visible = Arc::new(Mutex::new(true));
        }

        Self {
            listener_app_focus_status: None,
            listener_xcode_focus_status_change: None,
            preserve_content_visibility_was_visible,
            tauri_app_handle: app_handle.clone(),
        }
    }

    pub fn setup(&mut self) {
        // 1. Prepare variables to be moved into closure
        let preserve_content_visibility_was_visible_copy =
            self.preserve_content_visibility_was_visible.clone();
        let tauri_app_handle_copy = self.tauri_app_handle.clone();
        let listener_app_focus_status =
            self.tauri_app_handle
                .listen_global("StateEvent-AppFocusState", move |msg| {
                    if let Some(msg_s) = msg.payload() {
                        let parsed_msg: types::Event = serde_json::from_str(&msg_s).unwrap();

                        if let types::Event::AppFocusState(payload) = parsed_msg {
                            if !["app", "XCode"].contains(&&payload.current_app.name.as_str()) {
                                Self::hide_widget_preserve_content(
                                    tauri_app_handle_copy.clone(),
                                    &preserve_content_visibility_was_visible_copy,
                                );
                            }
                        }
                    }
                });
        self.listener_app_focus_status = Some(listener_app_focus_status);

        // Registering listener for Editor Focus
        // ==================================
        // 1. Prepare variables to be moved into closure
        let preserve_content_visibility_was_visible_copy =
            self.preserve_content_visibility_was_visible.clone();
        let tauri_app_handle_copy = self.tauri_app_handle.clone();

        // 2. Create listener
        let listener_xcode_focus_status_change =
            self.tauri_app_handle
                .listen_global("StateEvent-XCodeFocusStatusChange", move |msg| {
                    if let Some(msg_s) = msg.payload() {
                        let parsed_msg: types::Event = serde_json::from_str(&msg_s).unwrap();

                        if let types::Event::XCodeFocusStatusChange(payload) = parsed_msg {
                            if let models::XCodeFocusElement::Editor = payload.focus_element_change
                            {
                                if payload.is_in_focus {
                                    Self::show_widget_preserve_content(
                                        tauri_app_handle_copy.clone(),
                                        &preserve_content_visibility_was_visible_copy,
                                    );
                                }
                            }
                        }
                    }
                });
        self.listener_xcode_focus_status_change = Some(listener_xcode_focus_status_change);
    }

    fn hide_widget_preserve_content(app_handle: tauri::AppHandle, preserve_var: &Arc<Mutex<bool>>) {
        // Preserve content visibility before hiding it due to the widget being hidden
        let content_window = app_handle.get_window(&AppWindow::Content.to_string());
        if content_window.is_some() {
            let mut locked_val = preserve_var.lock().unwrap();
            *locked_val = content_window.unwrap().is_visible().unwrap();
        }

        cmd_close_window(app_handle.clone(), AppWindow::Widget);
        cmd_close_window(app_handle.clone(), AppWindow::Content);

        let _my_state = app_handle.state::<XCodeTwin>().get_state_global_app_focus();
        let _my_state2 = app_handle
            .state::<XCodeTwin>()
            .get_state_xcode_focus_state();
    }

    fn show_widget_preserve_content(app_handle: tauri::AppHandle, preserve_var: &Arc<Mutex<bool>>) {
        cmd_open_window(app_handle.clone(), AppWindow::Widget);

        // Restore content visibility after showing the widget
        let mut locked_val = preserve_var.lock().unwrap();
        if *locked_val {
            cmd_open_window(app_handle.clone(), AppWindow::Content);
            *locked_val = false;
        }

        let _my_state = app_handle.state::<XCodeTwin>().get_state_global_app_focus();
        let _my_state2 = app_handle
            .state::<XCodeTwin>()
            .get_state_xcode_focus_state();
    }
}

impl Drop for WindowStateMachine {
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
