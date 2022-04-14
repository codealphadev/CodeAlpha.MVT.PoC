use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::ax_interaction::{
    models::editor::{EditorWindowCreatedMessage, EditorWindowDestroyedMessage},
    AXEventXcode, AX_EVENT_XCODE_CHANNEL,
};

use super::{editor_window::EditorWindow, WidgetWindow};

pub struct WindowStateManager {
    tauri_app_handle: tauri::AppHandle,

    // List of listeners; stored to be able to safely remove them
    _open_editor_windows: Arc<Mutex<Vec<EditorWindow>>>,
    widget_window: Arc<Mutex<WidgetWindow>>,
}

impl WindowStateManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let _open_editor_windows: Arc<Mutex<Vec<EditorWindow>>> = Arc::new(Mutex::new(Vec::new()));

        // Register listener for xcode events
        let open_editors_move_copy = _open_editor_windows.clone();
        app_handle.listen_global(AX_EVENT_XCODE_CHANNEL, move |msg| {
            let axevent_xcode: AXEventXcode =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_xcode {
                AXEventXcode::EditorWindowCreated(msg) => {
                    Self::add_editor_window(&open_editors_move_copy, &msg);
                }
                AXEventXcode::EditorWindowDestroyed(msg) => {
                    Self::remove_editor_window(&open_editors_move_copy, &msg);
                }
                AXEventXcode::EditorClosed(_) => {
                    let mut editors_locked = open_editors_move_copy.lock().unwrap();
                    *editors_locked = Vec::new();
                }
                _ => {}
            }
        });

        Self {
            tauri_app_handle: app_handle.clone(),
            _open_editor_windows: _open_editor_windows.clone(),
            widget_window: Arc::new(Mutex::new(
                WidgetWindow::new(&app_handle, &_open_editor_windows).unwrap(),
            )),
        }
    }

    pub fn configure_windows(&mut self) {
        WidgetWindow::setup_widget_listeners(&self.tauri_app_handle, &mut self.widget_window);
        WidgetWindow::start_widget_visibility_control(
            &self.tauri_app_handle,
            &mut self.widget_window,
        );
    }

    fn add_editor_window(
        editor_window_list: &Arc<Mutex<Vec<EditorWindow>>>,
        created_msg: &EditorWindowCreatedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        // check if window is already contained in list of windows
        let window_exists = (*editor_list_locked)
            .iter()
            .any(|window| window.id == created_msg.id);

        if !window_exists {
            (*editor_list_locked).push(EditorWindow::new(created_msg));
        }
    }

    fn remove_editor_window(
        editor_window_list: &Arc<Mutex<Vec<EditorWindow>>>,
        destroyed_msg: &EditorWindowDestroyedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        let _ = &editor_list_locked.retain(|editor_window| {
            // returning false in Vec::retain() will remove the element from the vector
            if editor_window.id == destroyed_msg.id {
                false
            } else {
                true
            }
        });
    }
}
