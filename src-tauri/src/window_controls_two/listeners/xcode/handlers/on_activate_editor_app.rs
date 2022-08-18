use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::{EditorAppActivatedMessage, FocusedUIElement},
    window_controls_two::{config::AppWindow, WindowManager},
};

pub fn on_activate_editor_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    _activated_msg: &EditorAppActivatedMessage,
) -> Option<()> {
    let mut window_manager = window_manager.lock();
    window_manager.set_is_editor_focused(true);

    let mut is_textarea_focused = false;
    {
        let editor_window_list = &mut window_manager.editor_windows().lock();
        let editor_window = editor_window_list.get(&window_manager.focused_editor_window()?)?;

        if *editor_window.focused_ui_element()? == FocusedUIElement::Textarea {
            is_textarea_focused = true;
        }
    }

    if is_textarea_focused {
        window_manager.show_app_windows(AppWindow::shown_windows_on_focus_gained(), None);
    }

    Some(())
}
