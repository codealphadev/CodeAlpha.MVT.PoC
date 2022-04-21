use std::sync::{Arc, Mutex};

use crate::{
    ax_interaction::xcode::{get_xcode_editor_content, update_xcode_editor_content},
    window_controls::WidgetWindow,
};

/// This command contains too much accessibility logic - going to give us a harder time in the future. Need better design.
#[tauri::command]
pub fn cmd_search_and_replace(
    search_str: String,
    replace_str: String,
    widget_state: tauri::State<'_, Arc<Mutex<WidgetWindow>>>,
) {
    let widget_window = &*(widget_state.lock().unwrap());
    let editor_windows = &*(widget_window.editor_windows.lock().unwrap());

    if let Some(focused_editor_window_id) = widget_window.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows
            .iter()
            .find(|window| window.id == focused_editor_window_id)
        {
            let content = get_xcode_editor_content(editor_window.pid.try_into().unwrap());

            if let Ok(content) = content {
                if let Some(content_str) = content {
                    let content_str = content_str.replace(&search_str, &replace_str);
                    let _ = update_xcode_editor_content(
                        editor_window.pid.try_into().unwrap(),
                        &content_str,
                    );
                }
            }
        }
    }
}
