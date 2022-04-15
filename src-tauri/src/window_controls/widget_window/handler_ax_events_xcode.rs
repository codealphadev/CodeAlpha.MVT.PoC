use std::time::{Duration, Instant};

use crate::ax_interaction::models::editor::{
    EditorAppActivatedMessage, EditorAppClosedMessage, EditorAppDeactivatedMessage,
    EditorUIElementFocusedMessage, EditorWindowMovedMessage, EditorWindowResizedMessage,
};

use super::{
    widget_window::{
        HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS, HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS,
        XCODE_EDITOR_NAME,
    },
    WidgetWindow,
};

pub fn on_resize_editor_window(
    widget_props: &mut WidgetWindow,
    resize_msg: &EditorWindowResizedMessage,
) {
    let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

    for window in &mut *editor_list_locked {
        if window.id == resize_msg.id {
            window.update_window_dimensions(
                resize_msg.window_position,
                resize_msg.window_size,
                resize_msg.textarea_position,
                resize_msg.textarea_size,
            );

            // Reset hide timer after which the widget should be displayed again
            widget_props.hide_until_instant =
                Instant::now() + Duration::from_millis(HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS);

            break;
        }
    }
}

pub fn on_move_editor_window(
    widget_props: &mut WidgetWindow,
    moved_msg: &EditorWindowMovedMessage,
) {
    let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

    for window in &mut *editor_list_locked {
        if window.id == moved_msg.id {
            window.update_window_dimensions(
                moved_msg.window_position,
                moved_msg.window_size,
                None,
                None,
            );

            // Reset hide timer after which the widget should be displayed again
            widget_props.hide_until_instant =
                Instant::now() + Duration::from_millis(HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS);

            break;
        }
    }
}

/// Update EditorWindow to which of it's ui elements is currently in focus. Furthermore, also update
/// which of all open editor windows is currently in focus.
pub fn on_editor_ui_element_focus_change(
    widget_props: &mut WidgetWindow,
    focus_msg: &EditorUIElementFocusedMessage,
) {
    let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

    for window in &mut *editor_list_locked {
        if window.id == focus_msg.window_id {
            window.update_focused_ui_element(
                &focus_msg.focused_ui_element,
                focus_msg.textarea_position,
                focus_msg.textarea_size,
            );

            // Set which editor window is currently focused
            widget_props.currently_focused_editor_window = Some(window.id);

            break;
        }
    }
}

pub fn on_deactivate_editor_app(
    widget_props: &mut WidgetWindow,
    deactivated_msg: &EditorAppDeactivatedMessage,
) {
    if deactivated_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = false;
    }

    widget_props.delay_hide_until_instant =
        Instant::now() + Duration::from_millis(HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS);
}

pub fn on_close_editor_app(widget_props: &mut WidgetWindow, closed_msg: &EditorAppClosedMessage) {
    if closed_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = false;
    }
}

pub fn on_activate_editor_app(
    widget_props: &mut WidgetWindow,
    activated_msg: &EditorAppActivatedMessage,
) {
    if activated_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = true;
    }
}
