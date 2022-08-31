use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::editor::{EditorAppActivatedMessage, FocusedUIElement},
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_activate_editor_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    activated_msg: &EditorAppActivatedMessage,
) -> Option<()> {
    let mut window_manager = window_manager.lock();

    let is_textarea_focused;
    {
        let editor_window_list = &mut window_manager.editor_windows().lock();
        let editor_window = editor_window_list.get_mut(&activated_msg.window_uid)?;

        if *editor_window.focused_ui_element()? == FocusedUIElement::Textarea {
            is_textarea_focused = true;
            editor_window.check_and_update_dark_mode().ok();
        } else {
            is_textarea_focused = false;
        }
    }

    if is_textarea_focused {
        window_manager.show_app_windows(
            AppWindow::shown_on_focus_gained(),
            Some(activated_msg.window_uid),
        );
    }

    window_manager.set_focused_editor_window(Some(activated_msg.window_uid));

    Some(())
}
