use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{app_handle, CORE_ENGINE_ACTIVE_AT_STARTUP};

use super::{
    config::AppWindow,
    listeners::{app_listener, user_interaction_listener, xcode_listener},
    windows::{CodeOverlayWindow, EditorWindow, WidgetWindow},
};

pub static SUPPORTED_EDITORS: &[&str] = &["Xcode"];

pub type Uuid = usize;

#[derive(Clone, Debug)]
pub struct WindowManager {
    app_handle: tauri::AppHandle,

    /// HashMap of open editor windows.
    editor_windows: Arc<Mutex<HashMap<Uuid, EditorWindow>>>,

    // WidgetWindow
    widget_window: Arc<Mutex<WidgetWindow>>,

    // CodeOverlayWindow
    code_overlay_window: Arc<Mutex<CodeOverlayWindow>>,

    /// Identitfier of the currently focused editor window. Is None until the first window was focused.
    focused_editor_window: Option<Uuid>,

    /// Boolean saying if the currently focused application is an editor window.
    is_editor_focused: bool,

    /// Identitfier of the currently focused app window. Is None until the first window was focused.
    focused_app_window: Option<AppWindow>,

    /// Boolean saying if the currently focused application is our app.
    is_app_focused: bool,

    /// Boolean stating if the the core engine is active.
    is_core_engine_active: bool,
}

impl WindowManager {
    pub fn new() -> Result<Self, tauri::Error> {
        // Instantiate app windows. If this fails, the app will not work.
        let widget_window = Arc::new(Mutex::new(WidgetWindow::new()?));
        WidgetWindow::start_event_listeners(&widget_window);

        let code_overlay_window = Arc::new(Mutex::new(CodeOverlayWindow::new()?));
        CodeOverlayWindow::start_event_listeners(&code_overlay_window);

        Ok(Self {
            app_handle: app_handle(),
            editor_windows: Arc::new(Mutex::new(HashMap::new())),
            focused_editor_window: None,
            is_app_focused: false,
            is_editor_focused: false,
            focused_app_window: None,
            is_core_engine_active: CORE_ENGINE_ACTIVE_AT_STARTUP,
            widget_window,
            code_overlay_window,
        })
    }

    pub fn editor_windows(&self) -> &Arc<Mutex<HashMap<Uuid, EditorWindow>>> {
        &self.editor_windows
    }

    pub fn clear_editor_windows(&mut self, editor_name: &String) {
        let mut editor_windows = self.editor_windows.lock();
        editor_windows.retain(|_, editor_window| {
            if editor_window.editor_name() != editor_name {
                true
            } else {
                self.focused_editor_window = None;
                self.is_editor_focused = false;
                false
            }
        });
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

    pub fn is_core_engine_active(&self) -> bool {
        self.is_core_engine_active
    }

    pub fn set_is_core_engine_active(&mut self, is_core_engine_active: bool) {
        self.is_core_engine_active = is_core_engine_active;
    }

    pub fn set_focused_editor_window(&mut self, editor_window_hash: Uuid) {
        self.focused_editor_window = Some(editor_window_hash);
    }

    pub fn set_focused_app_window(&mut self, app_window: AppWindow) {
        self.focused_app_window = Some(app_window);
    }

    pub fn start_event_listeners(window_manager: &Arc<Mutex<WindowManager>>) {
        app_listener(window_manager);
        user_interaction_listener(window_manager);
        xcode_listener(window_manager);
    }
}
