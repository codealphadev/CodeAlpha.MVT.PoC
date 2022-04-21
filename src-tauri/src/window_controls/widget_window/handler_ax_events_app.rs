use crate::{
    ax_interaction::models::app::{AppContentActivationMessage, AppWindowMovedMessage},
    window_controls::AppWindow,
};

use super::WidgetWindow;

pub fn on_move_app_window(widget_props: &mut WidgetWindow, move_msg: &AppWindowMovedMessage) {
    let editor_windows = &mut *(widget_props.editor_windows.lock().unwrap());
    if let Some(focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows
            .iter_mut()
            .find(|window| window.id == focused_editor_window_id)
        {
            if move_msg.window == AppWindow::Widget {
                editor_window.update_widget_position(move_msg.window_position);
            }
        }
    }
}

pub fn on_toggle_content_window(
    widget_props: &mut WidgetWindow,
    toggle_msg: &AppContentActivationMessage,
) {
    let editor_windows = &mut *(widget_props.editor_windows.lock().unwrap());
    if let Some(focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows
            .iter_mut()
            .find(|window| window.id == focused_editor_window_id)
        {
            editor_window.update_content_window_state(&toggle_msg.activation_state);
        }
    }
}
