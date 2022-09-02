use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::editor::EditorWindowMinimizedMessage,
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_minimize_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    _minimize_msg: &EditorWindowMinimizedMessage,
) {
    let mut window_manager = window_manager.lock();

    window_manager.hide_app_windows(AppWindow::hidden_on_focus_lost());
}
