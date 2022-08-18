use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::{
        derive_xcode_textarea_dimensions, get_textarea_uielement,
        models::editor::EditorWindowResizedMessage,
    },
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls_two::{config::AppWindow, WindowManager},
};

pub fn on_resize_editor_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    resize_msg: &EditorWindowResizedMessage,
) -> Option<()> {
    let window_manager = &mut window_manager.lock();
    let editor_window_list = &mut window_manager.editor_windows().lock();

    let editor_window = editor_window_list.get_mut(&resize_msg.uielement_hash)?;

    let mut textarea_position = resize_msg.textarea_position;
    let mut textarea_size = resize_msg.textarea_size;

    // If the textarea dimensions are not set, attempt to derive them from the textarea element.
    if let Some(elem) = get_textarea_uielement(editor_window.pid()) {
        if let Ok((position, size)) = derive_xcode_textarea_dimensions(&elem) {
            textarea_position = Some(position);
            textarea_size = Some(size);
        }
    }

    editor_window.update_window_dimensions(
        LogicalPosition::from_tauri_LogicalPosition(&resize_msg.window_position),
        LogicalSize::from_tauri_LogicalSize(&resize_msg.window_size),
        unpack_logical_position_tauri(textarea_position),
        unpack_logical_size_tauri(textarea_size),
    );

    window_manager.temporarily_hide_app_windows(AppWindow::hidden_on_focus_lost());

    Some(())
}

fn unpack_logical_position_tauri(
    position: Option<tauri::LogicalPosition<f64>>,
) -> Option<LogicalPosition> {
    Some(LogicalPosition {
        x: position?.x,
        y: position?.y,
    })
}

fn unpack_logical_size_tauri(size: Option<tauri::LogicalSize<f64>>) -> Option<LogicalSize> {
    Some(LogicalSize {
        width: size?.width,
        height: size?.height,
    })
}
