use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorTextareaScrolledMessage,
    window_controls_two::WindowManager,
};

pub fn on_editor_textarea_scrolled(
    window_manager: &Arc<Mutex<WindowManager>>,
    _scrolled_msg: &EditorTextareaScrolledMessage,
) {
    todo!("WidgetWindow::temporary_hide_check_routine(&app_handle, widget_props, false, true)");
}
