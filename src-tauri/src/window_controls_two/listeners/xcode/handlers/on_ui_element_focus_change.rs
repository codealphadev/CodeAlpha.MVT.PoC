use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::{EditorUIElementFocusedMessage, FocusedUIElement},
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls_two::{config::AppWindow, WindowManager},
};

pub fn on_editor_ui_element_focus_change(
    window_manager: &Arc<Mutex<WindowManager>>,
    focus_msg: &EditorUIElementFocusedMessage,
) -> Option<()> {
    let window_manager = &mut window_manager.lock();

    {
        let editor_window_list = &mut window_manager.editor_windows().lock();

        let editor_window = editor_window_list.get_mut(&focus_msg.ui_elem_hash)?;
        editor_window.update_focused_ui_element(
            &focus_msg.focused_ui_element,
            unpack_logical_position_tauri(focus_msg.textarea_position),
            unpack_logical_size_tauri(focus_msg.textarea_size),
        );
    }

    let mut need_temporary_hide = false;
    if let Some(previously_focused_window_id) = window_manager.focused_editor_window() {
        if previously_focused_window_id != focus_msg.ui_elem_hash {
            if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
                // Need to temporarily hide our windows when the user switches between editor windows
                // This gives our windows time to gracefully update their positions and sizes.
                need_temporary_hide = true;
            } else {
                window_manager.hide_app_windows(AppWindow::hidden_windows_on_focus_lost());
            }
        } else {
            if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
                window_manager.show_app_windows(
                    AppWindow::shown_windows_on_focus_gained(),
                    Some(focus_msg.ui_elem_hash),
                );
            } else {
                window_manager.hide_app_windows(AppWindow::hidden_windows_on_focus_lost());
            }
        }
    } else {
        // Case: app was started while focus was already on a valid editor textarea.
        if focus_msg.focused_ui_element == FocusedUIElement::Textarea {
            window_manager.show_app_windows(
                AppWindow::shown_windows_on_focus_gained(),
                Some(focus_msg.ui_elem_hash),
            );
        }
    }

    // Set which editor window is currently focused
    window_manager.set_focused_editor_window(focus_msg.ui_elem_hash);

    if need_temporary_hide {
        todo!(
            "WidgetWindow::temporary_hide_check_routine(&app_handle, window_manager, true, true);"
        );
    }

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
