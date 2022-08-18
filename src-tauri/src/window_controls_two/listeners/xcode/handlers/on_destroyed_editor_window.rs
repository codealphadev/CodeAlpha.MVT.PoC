use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorWindowDestroyedMessage,
    window_controls_two::WindowManager,
};

pub fn on_destroyed_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    destroyed_msg: &EditorWindowDestroyedMessage,
) {
    let window_manager = window_manager.lock();

    let editor_window_list = &mut window_manager.editor_windows().lock();

    // Remove the new window from the list of editor windows.
    _ = &editor_window_list.remove(&destroyed_msg.uielement_hash);
}
