use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{app_handle, ax_interaction::models::editor::FocusedUIElement};

use super::{
    config::AppWindow,
    editor_window::EditorWindow,
    listeners::{app_listener, user_interaction_listener, xcode_listener},
};

pub static SUPPORTED_EDITORS: &[&str] = &["Xcode"];

pub type Uuid = usize;

#[derive(Clone, Debug)]
pub struct WindowManager {
    app_handle: tauri::AppHandle,

    /// HashMap of open editor windows.
    editor_windows: Arc<Mutex<HashMap<Uuid, EditorWindow>>>,

    /// Identitfier of the currently focused editor window. Is None until the first window was focused.
    focused_editor_window: Option<Uuid>,

    /// Boolean saying if the currently focused application is our app.
    is_app_focused: bool,

    /// Boolean saying if the currently focused application is an editor window.
    is_editor_focused: bool,

    /// Identitfier of the currently focused app window. Is None until the first window was focused.
    focused_app_window: Option<AppWindow>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            app_handle: app_handle(),
            editor_windows: Arc::new(Mutex::new(HashMap::new())),
            focused_editor_window: None,
            is_app_focused: false,
            is_editor_focused: false,
            focused_app_window: None,
        }
    }

    pub fn editor_windows(&self) -> &Arc<Mutex<HashMap<Uuid, EditorWindow>>> {
        &self.editor_windows
    }

    pub fn focused_editor_window(&self) -> Option<Uuid> {
        self.focused_editor_window
    }

    pub fn is_editor_focused(&self) -> bool {
        self.is_editor_focused
    }

    pub fn set_is_editor_focused(&mut self, is_editor_focused: bool) {
        self.is_editor_focused = is_editor_focused;
    }

    pub fn is_app_focused(&self) -> bool {
        self.is_app_focused
    }

    pub fn set_is_app_focused(&mut self, is_app_focused: bool) {
        self.is_app_focused = is_app_focused;
    }

    pub fn set_focused_editor_window(&mut self, editor_window_hash: Uuid) {
        self.focused_editor_window = Some(editor_window_hash);
    }

    pub fn start_event_listeners(window_manager: &Arc<Mutex<WindowManager>>) {
        app_listener(window_manager);
        user_interaction_listener(window_manager);
        xcode_listener(window_manager);
    }
}
