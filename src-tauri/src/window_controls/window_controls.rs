use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tauri::Manager;

use crate::{
    ax_interaction::{
        models::editor::{EditorWindowCreatedMessage, EditorWindowDestroyedMessage},
        AXEventReplit, AXEventXcode,
    },
    core_engine::events::{models::CoreActivationStatusMessage, EventUserInteraction},
    utils::messaging::ChannelList,
};

use super::{actions::open_window, config::AppWindow, editor_window::EditorWindow};

pub struct WindowControls {}

impl WindowControls {
    pub fn new(
        app_handle: &tauri::AppHandle,
        editor_windows: Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,
    ) -> Self {
        // Register listener for xcode events to create / remove editor
        let editor_windows_move_copy = editor_windows.clone();
        let handle_move_copy = app_handle.clone();
        app_handle.listen_global(ChannelList::AXEventXcode.to_string(), move |msg| {
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
                    let mut editors_locked = match editor_windows_move_copy.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    *editors_locked = HashMap::new();
                }
                AXEventXcode::EditorAppCodeSelected(msg) => {
                    handle_move_copy.emit_all("evt-repair-opened", msg).unwrap();
                    open_window(&handle_move_copy, AppWindow::Repair);
                }
                _ => {}
            }
        });

        // Register listener for Replit events to create / remove editor
        let editor_windows_move_copy = editor_windows.clone();
        app_handle.listen_global(ChannelList::AXEventReplit.to_string(), move |msg| {
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
                    let mut editors_locked = match editor_windows_move_copy.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    *editors_locked = HashMap::new();
                }
                _ => {}
            }
        });

        Self {}
    }

    fn add_editor_window(
        editor_window_list: &Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,
        created_msg: &EditorWindowCreatedMessage,
    ) {
        let mut editor_list_locked = match editor_window_list.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // check if window is already contained in list of windows
        if (*editor_list_locked).get(&created_msg.id).is_none() {
            (*editor_list_locked).insert(created_msg.id, EditorWindow::new(created_msg));
        }
    }

    fn remove_editor_window(
        editor_window_list: &Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,
        destroyed_msg: &EditorWindowDestroyedMessage,
    ) {
        let mut editor_list_locked = match editor_window_list.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let _ = &editor_list_locked.remove(&destroyed_msg.id);
    }
}

#[tauri::command]
pub fn cmd_toggle_app_activation(app_handle: tauri::AppHandle, app_active: bool) {
    EventUserInteraction::CoreActivationStatus(CoreActivationStatusMessage {
        engine_active: Some(app_active),
        active_feature: None,
    })
    .publish_to_tauri(&app_handle);
}
