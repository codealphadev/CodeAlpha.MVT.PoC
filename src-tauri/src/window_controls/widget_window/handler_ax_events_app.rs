use crate::{
    ax_interaction::{
        _is_currently_focused_app_editor,
        models::app::{AppContentActivationMessage, AppDeactivatedMessage, AppWindowMovedMessage},
    },
    window_controls::config::AppWindow,
};

use super::WidgetWindow;

pub fn on_move_app_window(widget_props: &mut WidgetWindow, move_msg: &AppWindowMovedMessage) {
    if move_msg.window != AppWindow::Widget {
        return;
    }

    let editor_windows = &mut *(match widget_props.editor_windows.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });
    if let Some(focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows.get_mut(&focused_editor_window_id) {
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
    let editor_windows = &mut *(match widget_props.editor_windows.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });
    if let Some(focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows.get_mut(&focused_editor_window_id) {
            editor_window.update_content_window_state(&toggle_msg.activation_state);
        }
    }
}

pub fn on_deactivate_app(
    widget_props: &mut WidgetWindow,
    _deactivated_msg: &AppDeactivatedMessage,
) {
    widget_props.is_app_focused = false;

    if let Some(is_focused_app_editor) = _is_currently_focused_app_editor() {
        if !is_focused_app_editor {
            WidgetWindow::hide_widget_routine(&widget_props.app_handle);
        }
    } else {
        WidgetWindow::hide_widget_routine(&widget_props.app_handle);
    }
}
