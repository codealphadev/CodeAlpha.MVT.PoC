use crate::{ax_interaction::models::app::AppWindowMovedMessage, window_controls::AppWindow};

use super::WidgetWindow;

pub fn on_move_app_window(widget_props: &mut WidgetWindow, move_msg: &AppWindowMovedMessage) {
    let editor_windows = &mut *(widget_props.editor_windows.lock().unwrap());
    if let Some(focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows
            .iter_mut()
            .find(|window| window.id == focused_editor_window_id)
        {
            if move_msg.window == AppWindow::Widget {
                editor_window.update_widget_position_through_dragging(move_msg.window_position);
            }
        }
    }
}
