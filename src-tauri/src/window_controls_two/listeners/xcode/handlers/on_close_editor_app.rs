use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorAppClosedMessage,
    window_controls_two::{window_manager::SUPPORTED_EDITORS, WindowManager},
};

pub fn on_close_editor_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    closed_msg: &EditorAppClosedMessage,
) {
    let mut window_manager = window_manager.lock();

    // Determine if we need to hide the widget because the editor app that is closed is the one that is focused.
    if let Some(focused_editor_window) = window_manager.focused_editor_window() {
        // Get the editor window that is currently focused.
        if let Some(editor_window) = window_manager
            .editor_windows()
            .lock()
            .get(&focused_editor_window)
        {
            // Check if the editor window that is currently focused is the one that was closed.
            if *editor_window.editor_name() == closed_msg.editor_name {
                todo!("WidgetWindow::hide_widget_routine(&widget_window.app_handle)");
            }
        }
    }

    window_manager.clear_editor_windows(&closed_msg.editor_name);
}
