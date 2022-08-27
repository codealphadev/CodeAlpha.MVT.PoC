use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::{
        is_currently_focused_app_our_app, models::editor::EditorAppDeactivatedMessage,
    },
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_deactivate_editor_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    _deactivated_msg: &EditorAppDeactivatedMessage,
) {
    let window_manager = window_manager.lock();

    if let Ok(is_focused_app_our_app) = is_currently_focused_app_our_app() {
        if !is_focused_app_our_app {
            window_manager.hide_app_windows(AppWindow::hidden_on_focus_lost());
        }
    } else {
        window_manager.hide_app_windows(AppWindow::hidden_on_focus_lost());
    }
}
