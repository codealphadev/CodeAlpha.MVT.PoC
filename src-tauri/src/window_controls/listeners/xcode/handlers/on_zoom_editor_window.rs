use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::{get_viewport_frame, models::editor::EditorTextareaZoomedMessage, GetVia},
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_zoom_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    zoom_msg: &EditorTextareaZoomedMessage,
) -> Option<()> {
    let window_manager = &mut window_manager.lock();
    let editor_window_list = &mut window_manager.editor_windows().try_lock()?;

    let editor_window = editor_window_list.get_mut(&zoom_msg.window_uid)?;

    let code_section_frame = get_viewport_frame(&GetVia::Pid(editor_window.pid())).ok()?;

    editor_window.update_textarea_dimensions(LogicalFrame {
        origin: LogicalPosition {
            x: code_section_frame.origin.x,
            y: code_section_frame.origin.y,
        },
        size: LogicalSize {
            width: code_section_frame.size.width,
            height: code_section_frame.size.height,
        },
    });

    window_manager.temporarily_hide_app_windows(AppWindow::hiddon_on_zoom_level_change());

    Some(())
}
