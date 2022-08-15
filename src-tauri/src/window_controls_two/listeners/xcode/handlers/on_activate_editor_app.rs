use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::{EditorAppActivatedMessage, FocusedUIElement},
    window_controls_two::WindowManager,
};

pub fn on_activate_editor_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    _activated_msg: &EditorAppActivatedMessage,
) -> Option<()> {
    let mut window_manager = window_manager.lock();
    window_manager.set_is_editor_focused(true);

    let editor_window_list = &mut window_manager.editor_windows().lock();
    let editor_window = editor_window_list.get(&window_manager.focused_editor_window()?)?;

    if *editor_window.focused_ui_element()? == FocusedUIElement::Textarea {
        todo!(
            "WidgetWindow::show_widget_routine(
            &widget_props.app_handle,
            widget_props,
            &mut editor_list_locked,
        )"
        )
    }

    Some(())
}
