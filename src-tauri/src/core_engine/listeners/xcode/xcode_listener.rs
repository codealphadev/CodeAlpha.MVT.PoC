use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, core_engine::CoreEngine, platform::macos::AXEventXcode,
    utils::messaging::ChannelList,
};

use super::handlers::*;

pub fn xcode_listener(core_engine: &Arc<Mutex<CoreEngine>>) {
    app_handle().listen_global(ChannelList::AXEventXcode.to_string(), {
        let core_engine = core_engine.clone();
        move |msg| {
            let axevent_xcode: AXEventXcode =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_xcode {
                AXEventXcode::EditorWindowMoved(msg) => {
                    _ = on_editor_window_moved(&core_engine, &msg);
                }
                AXEventXcode::EditorWindowResized(msg) => {
                    _ = on_editor_window_resized(&core_engine, &msg);
                }
                AXEventXcode::EditorTextareaScrolled(msg) => {
                    _ = on_editor_textarea_scrolled(&core_engine, &msg);
                }
                AXEventXcode::EditorTextareaZoomed(msg) => {
                    _ = on_editor_textarea_zoomed(&core_engine, &msg);
                }
                AXEventXcode::EditorTextareaContentChanged(msg) => {
                    _ = on_text_content_changed(&core_engine, &msg);
                }
                AXEventXcode::EditorTextareaSelectedTextChanged(msg) => {
                    _ = on_selected_text_changed(&core_engine, &msg);
                }
                AXEventXcode::EditorAppClosed(_) => {
                    _ = on_close_editor_app(&core_engine);
                }
                AXEventXcode::EditorWindowCreated(_) => {
                    // We don't do anything because we don't keep track of open windows, here we are only
                    // interested in the displayed document
                }
                AXEventXcode::EditorWindowDestroyed(msg) => {
                    _ = on_editor_window_destroyed(&core_engine, &msg);
                }
                AXEventXcode::EditorUIElementFocused(msg) => {
                    _ = on_editor_focused_uielement_changed(&core_engine, &msg);
                }
                AXEventXcode::EditorShortcutPressed(msg) => {
                    _ = on_editor_shortcut_pressed(&core_engine, &msg);
                }
                AXEventXcode::EditorAppActivated(msg) => {
                    _ = on_app_activated(&core_engine, &msg);
                }
                AXEventXcode::EditorAppDeactivated(_) => {
                    // Do nothing
                }
            }
        }
    });
}
