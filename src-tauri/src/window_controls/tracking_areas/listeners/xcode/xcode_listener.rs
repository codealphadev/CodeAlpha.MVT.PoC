use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, platform::macos::AXEventXcode, utils::messaging::ChannelList,
    window_controls::TrackingAreasManager,
};

use super::handlers::on_move_editor_window;

pub fn xcode_listener(tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>) {
    app_handle().listen_global(ChannelList::AXEventXcode.to_string(), {
        let tracking_area_manager_arc = (tracking_area_manager_arc).clone();
        move |msg| {
            let axevent_xcode: AXEventXcode =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_xcode {
                AXEventXcode::EditorUIElementFocused(_) => {
                    // Do Nothing here (yet)
                }
                AXEventXcode::EditorWindowCreated(_) => {
                    // Do Nothing here (yet)
                }
                AXEventXcode::EditorWindowDestroyed(_) => {
                    // Do Nothing here (yet)
                }
                AXEventXcode::EditorWindowResized(_) => {
                    // Do Nothing here (yet)
                }
                AXEventXcode::EditorWindowMoved(msg) => {
                    on_move_editor_window(&tracking_area_manager_arc, &msg)
                }
                AXEventXcode::EditorAppActivated(_) => {
                    // Do Nothing here (yet)
                }
                AXEventXcode::EditorAppDeactivated(_) => {
                    // Do Nothing here (yet)
                }
                AXEventXcode::EditorAppClosed(_) => {
                    // Do Nothing here (yet)
                }
                AXEventXcode::EditorTextareaScrolled(_) => {
                    // Do Nothing here
                }
                AXEventXcode::EditorTextareaZoomed(_) => {
                    // Do Nothing here (yet)
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
                AXEventXcode::EditorWindowMinimized(_) => {
                    // Do Nothing here (yet)
                }
            }
        }
    });
}
