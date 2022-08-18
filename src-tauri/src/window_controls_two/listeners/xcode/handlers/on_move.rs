use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorWindowMovedMessage,
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls_two::{config::AppWindow, WindowManager},
};

pub fn on_move_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    moved_msg: &EditorWindowMovedMessage,
) -> Option<()> {
    let window_manager = &mut window_manager.lock();
    let editor_window_list = &mut window_manager.editor_windows().lock();

    let editor_window = editor_window_list.get_mut(&moved_msg.uielement_hash)?;

    editor_window.update_window_dimensions(
        LogicalPosition::from_tauri_LogicalPosition(&moved_msg.window_position),
        LogicalSize::from_tauri_LogicalSize(&moved_msg.window_size),
        None,
        None,
    );

    window_manager.temporarily_hide_app_windows(AppWindow::hidden_on_focus_lost());

    Some(())
}
