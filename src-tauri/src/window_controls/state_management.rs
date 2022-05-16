use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tauri::Manager;

use crate::ax_interaction::{
    models::editor::{EditorWindowCreatedMessage, EditorWindowDestroyedMessage},
    AXEventReplit, AXEventXcode, AX_EVENT_REPLIT_CHANNEL, AX_EVENT_XCODE_CHANNEL,
};

use super::{editor_window::EditorWindow, WidgetWindow};

#[allow(dead_code)]
pub struct WindowStateManager {
    tauri_app_handle: tauri::AppHandle,

    // List of listeners; stored to be able to safely remove them
    open_editor_windows: Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,
    widget_window: Arc<Mutex<WidgetWindow>>,
}

impl WindowStateManager {
    pub fn new(
        app_handle: &tauri::AppHandle,
        editor_windows: Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,
        widget_window: Arc<Mutex<WidgetWindow>>,
    ) -> Self {
        // Register listener for xcode events to create / remove editor
        let editor_windows_move_copy = editor_windows.clone();
        app_handle.listen_global(AX_EVENT_XCODE_CHANNEL, move |msg| {
            let axevent_xcode: AXEventXcode =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_xcode {
                AXEventXcode::EditorWindowCreated(msg) => {
                    Self::add_editor_window(&editor_windows_move_copy, &msg);
                }
                AXEventXcode::EditorWindowDestroyed(msg) => {
                    Self::remove_editor_window(&editor_windows_move_copy, &msg);
                }
                AXEventXcode::EditorAppClosed(_) => {
                    let mut editors_locked = editor_windows_move_copy.lock().unwrap();
                    *editors_locked = HashMap::new();
                }
                _ => {}
            }
        });

        // Register listener for Replit events to create / remove editor
        let editor_windows_move_copy = editor_windows.clone();
        app_handle.listen_global(AX_EVENT_REPLIT_CHANNEL, move |msg| {
            let axevent_replit: AXEventReplit =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_replit {
                AXEventReplit::EditorWindowCreated(msg) => {
                    Self::add_editor_window(&editor_windows_move_copy, &msg);
                }
                AXEventReplit::EditorWindowDestroyed(msg) => {
                    Self::remove_editor_window(&editor_windows_move_copy, &msg);
                }
                AXEventReplit::EditorAppClosed(_) => {
                    let mut editors_locked = editor_windows_move_copy.lock().unwrap();
                    *editors_locked = HashMap::new();
                }
                _ => {}
            }
        });

        Self {
            tauri_app_handle: app_handle.clone(),
            open_editor_windows: editor_windows.clone(),
            widget_window: widget_window.clone(),
        }
    }

    fn add_editor_window(
        editor_window_list: &Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,
        created_msg: &EditorWindowCreatedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        // check if window is already contained in list of windows
        if (*editor_list_locked).get(&created_msg.id).is_none() {
            (*editor_list_locked).insert(created_msg.id, EditorWindow::new(created_msg));
        }
    }

    fn remove_editor_window(
        editor_window_list: &Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,
        destroyed_msg: &EditorWindowDestroyedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        let _ = &editor_list_locked.remove(&destroyed_msg.id);
    }
}
