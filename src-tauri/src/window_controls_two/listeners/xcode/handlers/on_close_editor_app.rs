use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorAppClosedMessage,
    window_controls_two::{window_manager::SUPPORTED_EDITORS, WindowManager},
};

pub fn on_close_editor_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    closed_msg: &EditorAppClosedMessage,
) {
    let window_manager = window_manager.lock();

    if SUPPORTED_EDITORS.contains(&closed_msg.editor_name.as_str()) {
        todo!("WidgetWindow::hide_widget_routine(&widget_window.app_handle)");
    }
}
