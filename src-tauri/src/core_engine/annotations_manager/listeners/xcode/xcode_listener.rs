use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    core_engine::annotations_manager::{AnnotationsManager, AnnotationsManagerTrait},
    platform::macos::AXEventXcode,
    utils::messaging::ChannelList,
};

pub fn xcode_listener(annotations_manager_arc: &Arc<Mutex<AnnotationsManager>>) {
    app_handle().listen_global(ChannelList::AXEventXcode.to_string(), {
        let annotations_manager = annotations_manager_arc.clone();
        move |msg| {
            let axevent_xcode: AXEventXcode =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_xcode {
                // Resizing the editor window can cause lines to wrap / unwrap, which can cause annotations to change their position
                AXEventXcode::EditorWindowResized(msg) => annotations_manager
                    .lock()
                    .recompute_annotations(msg.window_uid),

                // Scrolling does not reposition annotation but formerly invisible annotations may become visible
                AXEventXcode::EditorTextareaScrolled(msg) => annotations_manager
                    .lock()
                    .update_annotations(msg.window_uid),

                // Zooming in can cause lines to wrap, which can cause annotations to change their position
                AXEventXcode::EditorTextareaZoomed(msg) => annotations_manager
                    .lock()
                    .recompute_annotations(msg.window_uid),

                // Triggered by re-focusing a editor textarea, because this could also mean a switch between different editors, we need to recompute all annotations
                AXEventXcode::EditorUIElementFocused(msg) => {
                    if let Some(window_uid) = msg.window_uid {
                        annotations_manager.lock().recompute_annotations(window_uid)
                    }
                }
                AXEventXcode::EditorAppClosed(_) => annotations_manager.lock().reset(),
                AXEventXcode::EditorWindowDestroyed(msg) => annotations_manager
                    .lock()
                    .remove_annotation_job_group_of_editor_window(msg.window_uid),

                // annotations are in local coordinates, moving the window does not affect them -> Do Nothing
                AXEventXcode::EditorWindowMoved(_) => {}
                AXEventXcode::EditorTextareaSelectedTextChanged(_) => {}
                AXEventXcode::EditorTextareaContentChanged(_) => {}
                AXEventXcode::EditorShortcutPressed(_) => {}
                AXEventXcode::EditorAppActivated(_) => {}
                AXEventXcode::EditorAppDeactivated(_) => {}
                AXEventXcode::EditorWindowMinimized(_) => {}
                AXEventXcode::EditorWindowCreated(_) => {}
            }
        }
    });
}
