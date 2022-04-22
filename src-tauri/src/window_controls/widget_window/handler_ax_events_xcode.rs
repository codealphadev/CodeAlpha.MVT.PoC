use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::ax_interaction::{
    is_currently_focused_app_our_app,
    models::editor::{
        EditorAppActivatedMessage, EditorAppClosedMessage, EditorAppDeactivatedMessage,
        EditorUIElementFocusedMessage, EditorWindowMovedMessage, EditorWindowResizedMessage,
        FocusedUIElement,
    },
};

use super::{
    widget_window::{
        self, hide_widget_routine, show_widget_routine, temporary_hide_check_routine,
        HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS, HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS,
        XCODE_EDITOR_NAME,
    },
    WidgetWindow,
};

pub fn on_resize_editor_window(
    widget_props: &mut WidgetWindow,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
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

            //temporary_hide_check_routine(&widget_props.app_handle, widget_arc);

            break;
        }
    }
}

pub fn on_move_editor_window(
    widget_props: &mut WidgetWindow,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
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

            //temporary_hide_check_routine(&widget_props.app_handle, widget_arc);

            break;
        }
    }
}

/// Update EditorWindow to which of it's ui elements is currently in focus. Furthermore, also update
/// which of all open editor windows is currently in focus.
pub fn on_editor_ui_element_focus_change(
    widget_props: &mut WidgetWindow,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    focus_msg: &EditorUIElementFocusedMessage,
) {
    let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

    println!(
        "on_editor_ui_element_focus_change: editor_window_id: {}, focused_ui_element: {:?}",
        focus_msg.window_id, focus_msg.focused_ui_element
    );
    // Update the focused ui element on the corresponding editor window instance.
    if let Some(editor_window) = editor_list_locked
        .iter_mut()
        .find(|window| window.id == focus_msg.window_id)
    {
        editor_window.update_focused_ui_element(
            &focus_msg.focused_ui_element,
            focus_msg.textarea_position,
            focus_msg.textarea_size,
        );
    } else {
        return;
    }

    if let Some(previously_focused_window_id) = widget_props.currently_focused_editor_window {
        if previously_focused_window_id != focus_msg.window_id {
            if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
                //temporary_hide_check_routine(&widget_props.app_handle, widget_arc);
            } else {
                hide_widget_routine(
                    &widget_props.app_handle,
                    widget_props,
                    &mut editor_list_locked,
                )
            }
        } else {
            if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
                show_widget_routine(
                    &widget_props.app_handle,
                    widget_props,
                    &mut editor_list_locked,
                )
            } else {
                hide_widget_routine(
                    &widget_props.app_handle,
                    widget_props,
                    &mut editor_list_locked,
                )
            }
        }
    }

    // Set which editor window is currently focused
    widget_props.currently_focused_editor_window = Some(focus_msg.window_id);
    widget_props.is_xcode_focused = true;
}

pub fn on_deactivate_editor_app(
    widget_props: &mut WidgetWindow,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    deactivated_msg: &EditorAppDeactivatedMessage,
) {
    if deactivated_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = false;
    }

    if let Some(is_focused_app_our_app) = is_currently_focused_app_our_app() {
        if !is_focused_app_our_app {
            let widget_window = &*(widget_arc.lock().unwrap());
            let editor_windows = &mut *(widget_window.editor_windows.lock().unwrap());
            hide_widget_routine(&widget_window.app_handle, &widget_window, editor_windows);
        }
    }
}

pub fn on_close_editor_app(
    widget_props: &mut WidgetWindow,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    closed_msg: &EditorAppClosedMessage,
) {
    if closed_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = false;

        let widget_window = &*(widget_arc.lock().unwrap());
        let editor_windows = &mut *(widget_window.editor_windows.lock().unwrap());
        hide_widget_routine(&widget_window.app_handle, &widget_window, editor_windows);
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
