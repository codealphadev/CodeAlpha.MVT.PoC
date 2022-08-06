use std::sync::{Arc, Mutex};

use crate::ax_interaction::{
    derive_xcode_textarea_dimensions, get_textarea_uielement, is_currently_focused_app_our_app,
    models::editor::{
        EditorAppActivatedMessage, EditorAppClosedMessage, EditorAppDeactivatedMessage,
        EditorTextareaScrolledMessage, EditorUIElementFocusedMessage, EditorWindowMovedMessage,
        EditorWindowResizedMessage, FocusedUIElement,
    },
};

use super::{widget_window::SUPPORTED_EDITORS, WidgetWindow};

pub fn on_resize_editor_window(
    app_handle: &tauri::AppHandle,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    resize_msg: &EditorWindowResizedMessage,
) {
    {
        let widget_props = &mut *(match widget_arc.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        });
        let mut editor_list_locked = match widget_props.editor_windows.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some(editor_window) = editor_list_locked.get_mut(&resize_msg.id) {
            let mut textarea_position = resize_msg.textarea_position;
            let mut textarea_size = resize_msg.textarea_size;

            // If the textarea dimensions are not set, attempt to derive them from the textarea element.
            if let Some(elem) = get_textarea_uielement(editor_window.pid) {
                if let Ok((position, size)) = derive_xcode_textarea_dimensions(&elem) {
                    textarea_position = Some(position);
                    textarea_size = Some(size);
                }
            }

            editor_window.update_window_dimensions(
                resize_msg.window_position,
                resize_msg.window_size,
                textarea_position,
                textarea_size,
            );
        } else {
            return;
        }
    }

    WidgetWindow::temporary_hide_check_routine(&app_handle, widget_arc, true, true);
}

pub fn on_move_editor_window(
    app_handle: &tauri::AppHandle,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    moved_msg: &EditorWindowMovedMessage,
) {
    {
        let widget_props = &mut *(match widget_arc.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        });
        let mut editor_list_locked = match widget_props.editor_windows.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some(editor_window) = editor_list_locked.get_mut(&moved_msg.id) {
            editor_window.update_window_dimensions(
                moved_msg.window_position,
                moved_msg.window_size,
                None,
                None,
            );
        } else {
            return;
        }
    }

    WidgetWindow::temporary_hide_check_routine(&app_handle, widget_arc, true, true);
}

/// Update EditorWindow to which of it's ui elements is currently in focus. Furthermore, also update
/// which of all open editor windows is currently in focus.
pub fn on_editor_ui_element_focus_change(
    app_handle: &tauri::AppHandle,
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    focus_msg: &EditorUIElementFocusedMessage,
) {
    // Introduce this boolean to conveniently wrap subsequent logic in own block to have
    // mutex drop at the end.
    let mut need_temporary_hide = false;

    {
        let widget_props = &mut *(match widget_arc.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        });
        let mut editor_list_locked = match widget_props.editor_windows.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Update the focused ui element on the corresponding editor window instance.
        if let Some(editor_window) = editor_list_locked.get_mut(&focus_msg.window_id) {
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
                    // Need to temporarily hide our windows when the user switches between editor windows
                    // This gives our windows time to gracefully update their positions and sizes.
                    need_temporary_hide = true;
                } else {
                    WidgetWindow::hide_widget_routine(&widget_props.app_handle)
                }
            } else {
                if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
                    WidgetWindow::show_widget_routine(
                        &widget_props.app_handle,
                        widget_props,
                        &mut editor_list_locked,
                    )
                } else {
                    WidgetWindow::hide_widget_routine(&widget_props.app_handle)
                }
            }
        } else {
            // Case: app was started while focus was already on a valid editor textarea.
            if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
                WidgetWindow::show_widget_routine(
                    &widget_props.app_handle,
                    widget_props,
                    &mut editor_list_locked,
                )
            }
        }

        // Set which editor window is currently focused
        widget_props.currently_focused_editor_window = Some(focus_msg.window_id);
    }

    if need_temporary_hide {
        WidgetWindow::temporary_hide_check_routine(&app_handle, widget_arc, true, true);
    }
}

pub fn on_deactivate_editor_app(
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    _deactivated_msg: &EditorAppDeactivatedMessage,
) {
    let widget_window = &mut *(match widget_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    widget_window.is_editor_focused = false;

    if let Some(is_focused_app_our_app) = is_currently_focused_app_our_app() {
        if !is_focused_app_our_app {
            WidgetWindow::hide_widget_routine(&widget_window.app_handle);
        }
    } else {
        WidgetWindow::hide_widget_routine(&widget_window.app_handle);
    }
}

pub fn on_close_editor_app(
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    closed_msg: &EditorAppClosedMessage,
) {
    let widget_window = &mut *(match widget_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if SUPPORTED_EDITORS.contains(&closed_msg.editor_name.as_str()) {
        WidgetWindow::hide_widget_routine(&widget_window.app_handle);
    }
}

pub fn on_activate_editor_app(
    widget_arc: &Arc<Mutex<WidgetWindow>>,
    _activated_msg: &EditorAppActivatedMessage,
) {
    let widget_props = &mut *(match widget_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });
    let editor_list_locked = match widget_props.editor_windows.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    widget_props.is_editor_focused = true;

    // Check if focused ui element of the currently focused editor window is textarea.
    if let Some(currently_focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_list_locked.get(&currently_focused_editor_window_id) {
            if let Some(focused_ui_element) = &editor_window.focused_ui_element {
                if *focused_ui_element == FocusedUIElement::Textarea {
                    WidgetWindow::show_widget_routine(
                        &widget_props.app_handle,
                        widget_props,
                        &editor_list_locked,
                    )
                }
            }
        }
    }
}

pub fn on_editor_textarea_scrolled(
    app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
    _scrolled_msg: &EditorTextareaScrolledMessage,
) {
    WidgetWindow::temporary_hide_check_routine(&app_handle, widget_props, false, true);
}
