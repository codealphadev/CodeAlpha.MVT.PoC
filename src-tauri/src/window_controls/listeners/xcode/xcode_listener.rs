use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, ax_interaction::AXEventXcode, utils::messaging::ChannelList,
    window_controls::WindowManager,
};

use super::handlers::{
    on_activate_editor_app, on_close_editor_app, on_created_editor_window,
    on_deactivate_editor_app, on_destroyed_editor_window, on_editor_ui_element_focus_change,
    on_move_editor_window, on_resize_editor_window, on_scroll_editor_window, on_zoom_editor_window,
};

pub fn xcode_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    let window_manager_move_copy = (window_manager).clone();
    app_handle().listen_global(ChannelList::AXEventXcode.to_string(), move |msg| {
        let axevent_xcode: AXEventXcode = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_xcode {
            AXEventXcode::EditorUIElementFocused(msg) => {
                on_editor_ui_element_focus_change(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorWindowCreated(msg) => {
                on_created_editor_window(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorWindowDestroyed(msg) => {
                on_destroyed_editor_window(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorWindowResized(msg) => {
                on_resize_editor_window(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorWindowMoved(msg) => {
                on_move_editor_window(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorAppActivated(msg) => {
                on_activate_editor_app(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorAppDeactivated(msg) => {
                on_deactivate_editor_app(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorAppClosed(msg) => {
                on_close_editor_app(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorAppCodeSelected(_) => {
                // Do Nothing here, DEPRECATED MESSAGE
            }
            AXEventXcode::EditorTextareaScrolled(_) => {
                on_scroll_editor_window(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorTextareaZoomed(msg) => {
                on_zoom_editor_window(&window_manager_move_copy, &msg);
            }
            AXEventXcode::EditorTextareaContentChanged(_) => {
                // Do Nothing here
            }
            AXEventXcode::EditorTextareaSelectedTextChanged(_) => {
                // Do Nothing here
            }
            AXEventXcode::EditorShortcutPressed(_) => {
                // Do Nothing here
            }
        }
    });
}
