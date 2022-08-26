use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::app::AppWindowMovedMessage, utils::geometry::LogicalPosition,
    window_controls::config::AppWindow, window_controls::WindowManager,
};

pub fn on_move_app_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    move_msg: &AppWindowMovedMessage,
) -> Option<()> {
    if move_msg.window == AppWindow::Widget {
        let window_manager = window_manager.lock();
        let focused_editor_window = window_manager.focused_editor_window()?;

        window_manager
            .editor_windows()
            .lock()
            .get_mut(&focused_editor_window)?
            .update_widget_position(LogicalPosition {
                x: move_msg.window_position.x,
                y: move_msg.window_position.y,
            });
    }

    Some(())
}
