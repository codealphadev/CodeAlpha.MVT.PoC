use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::{
        derive_xcode_textarea_dimensions, get_textarea_uielement,
        models::editor::EditorTextareaZoomedMessage,
    },
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_zoom_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    zoom_msg: &EditorTextareaZoomedMessage,
) -> Option<()> {
    let window_manager = &mut window_manager.lock();
    let editor_window_list = &mut window_manager.editor_windows().try_lock()?;

    let editor_window = editor_window_list.get_mut(&zoom_msg.uielement_hash)?;

    let textarea =
        derive_xcode_textarea_dimensions(&get_textarea_uielement(editor_window.pid())?).ok()?;

    editor_window.update_textarea_dimensions(LogicalFrame {
        origin: LogicalPosition {
            x: textarea.0.x,
            y: textarea.0.y,
        },
        size: LogicalSize {
            width: textarea.1.width,
            height: textarea.1.height,
        },
    });

    window_manager.temporarily_hide_app_windows(AppWindow::hidden_on_focus_lost());

    Some(())
}
