use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::{models::app::AppWindowMovedMessage, pressed_mouse_buttons},
    utils::geometry::LogicalPosition,
    window_controls::config::AppWindow,
    window_controls::WindowManager,
};

pub fn on_move_app_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    move_msg: &AppWindowMovedMessage,
) -> Option<()> {
    match move_msg.window {
        AppWindow::Settings => {
            // Do Nothing, for now.
        }
        AppWindow::Analytics => {
            // Do Nothing, for now.
        }
        AppWindow::Widget => {
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
        AppWindow::Content => {
            // Do Nothing, for now.
        }
        AppWindow::Explain => {
            let window_manager = window_manager.lock();

            // For this window, we only want to register movement caused by dragging the window
            // Therefore we check if a mouse button is pressed.
            if pressed_mouse_buttons().is_some() {
                window_manager.update_app_windows(
                    vec![AppWindow::Explain],
                    None,
                    Some(LogicalPosition::from_tauri_LogicalPosition(
                        &move_msg.window_position,
                    )),
                    None,
                )?;
            }
        }
        AppWindow::CodeOverlay => {
            // Do Nothing, for now.
        }
        AppWindow::Main => {
            // Do Nothing, for now.
        }
    }

    Some(())
}
