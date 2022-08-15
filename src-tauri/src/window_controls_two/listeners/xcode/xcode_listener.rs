use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, ax_interaction::AXEventXcode, utils::messaging::ChannelList,
    window_controls_two::WindowManager,
};

pub fn xcode_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    let window_manager_move_copy = (window_manager).clone();
    app_handle().listen_global(ChannelList::AXEventXcode.to_string(), move |msg| {
        let axevent_xcode: AXEventXcode = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_xcode {
            AXEventXcode::EditorUIElementFocused(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorWindowCreated(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorWindowDestroyed(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorWindowResized(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorWindowMoved(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorAppActivated(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorAppDeactivated(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorAppClosed(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorAppCodeSelected(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorTextareaScrolled(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorTextareaZoomed(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorTextareaContentChanged(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorTextareaSelectedTextChanged(msg) => {
                // Do Nothing
            }
            AXEventXcode::EditorShortcutPressed(msg) => {
                // Do Nothing
            }
        }
    });
}
