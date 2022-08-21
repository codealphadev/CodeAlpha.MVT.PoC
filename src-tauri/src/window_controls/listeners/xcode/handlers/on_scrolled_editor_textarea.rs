use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorTextareaScrolledMessage,
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_editor_textarea_scrolled(
    window_manager: &Arc<Mutex<WindowManager>>,
    _scrolled_msg: &EditorTextareaScrolledMessage,
) {
    let window_manager = window_manager.lock();

    window_manager.temporarily_hide_app_windows(AppWindow::hidden_on_scroll_event());
}
