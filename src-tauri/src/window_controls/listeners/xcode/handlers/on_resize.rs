use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::{get_code_section_frame, models::editor::EditorWindowResizedMessage, GetVia},
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_resize_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    resize_msg: &EditorWindowResizedMessage,
) -> Option<()> {
    let window_manager = &mut window_manager.lock();
    let editor_window_list = &mut window_manager.editor_windows().try_lock()?;

    let editor_window = editor_window_list.get_mut(&resize_msg.uielement_hash)?;

    let mut textarea_position = resize_msg.textarea_position;
    let mut textarea_size = resize_msg.textarea_size;

    // If the textarea dimensions are not set, attempt to derive them from the textarea element.
    if let Ok(code_section_frame) = get_code_section_frame(GetVia::Pid(editor_window.pid())) {
        textarea_position = Some(code_section_frame.origin.as_tauri_LogicalPosition());
        textarea_size = Some(code_section_frame.size.as_tauri_LogicalSize());
    };

    editor_window.update_window_and_textarea_dimensions(
        LogicalFrame {
            origin: LogicalPosition::from_tauri_LogicalPosition(&resize_msg.window_position),
            size: LogicalSize::from_tauri_LogicalSize(&resize_msg.window_size),
        },
        LogicalFrame {
            origin: LogicalPosition {
                x: textarea_position?.x,
                y: textarea_position?.y,
            },
            size: LogicalSize {
                width: textarea_size?.width,
                height: textarea_size?.height,
            },
        },
    );

    window_manager.temporarily_hide_app_windows(AppWindow::hidden_on_focus_lost());

    Some(())
}
