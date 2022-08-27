use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::editor::EditorWindowMovedMessage,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_move_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    moved_msg: &EditorWindowMovedMessage,
) -> Option<()> {
    let window_manager = &mut window_manager.lock();
    // If deadlock, discard message. TODO: Rethink logic.
    let editor_window_list = &mut window_manager.editor_windows().try_lock()?;

    let editor_window = editor_window_list.get_mut(&moved_msg.window_uid)?;

    editor_window.update_window_dimensions(LogicalFrame {
        origin: LogicalPosition::from_tauri_LogicalPosition(&moved_msg.window_position),
        size: LogicalSize::from_tauri_LogicalSize(&moved_msg.window_size),
    });

    window_manager.temporarily_hide_app_windows(AppWindow::hidden_on_focus_lost());

    Some(())
}
