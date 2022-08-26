use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::{
        currently_focused_ui_element, get_textarea_frame,
        models::editor::EditorTextareaScrolledMessage,
    },
    window_controls::{windows::EditorWindow, WindowManager},
};

pub fn on_scroll_editor_window(
    _window_manager: &Arc<Mutex<WindowManager>>,
    scroll_msg: &EditorTextareaScrolledMessage,
) -> Option<()> {
    let document_frame = get_textarea_frame(&currently_focused_ui_element().ok()?).ok()?;

    EditorWindow::update_code_document_frame(document_frame);

    return Some(());
}
