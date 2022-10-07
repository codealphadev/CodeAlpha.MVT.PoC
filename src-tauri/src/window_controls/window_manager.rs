use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use tracing::debug;

use crate::{
    app_handle,
    platform::macos::models::viewport::ViewportPropertiesUpdateMessage,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

use super::{
    config::AppWindow,
    events::{
        models::app_window::{HideAppWindowMessage, ShowAppWindowMessage},
        EventWindowControls,
    },
    listeners::{
        app_listener, rule_execution_listener, user_interaction_listener, viewport_update_listener,
        xcode_listener,
    },
    models::app_window::UpdateAppWindowMessage,
    windows::{CodeOverlayWindow, EditorWindow, ExplainWindow, MainWindow, WidgetWindow},
    TrackingAreasManager,
};

pub static HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS: u64 = 200;

pub type Uid = usize;

#[derive(Debug)]
pub struct WindowManager {
    app_handle: tauri::AppHandle,

    /// HashMap of open editor windows.
    editor_windows: Arc<Mutex<HashMap<Uid, EditorWindow>>>,

    /// Identitfier of the currently focused editor window. Is None until the first window was focused.
    focused_editor_window: Arc<Mutex<Option<Uid>>>,

    /// Identitfier of the currently focused app window. Is None until the first window was focused.
    focused_app_window: Option<AppWindow>,

    /// Boolean stating if the the core engine is active.
    is_core_engine_active: bool,

    // A timer instance used to temporarily hide our windows
    temporarily_hide_until: Arc<Mutex<std::time::Instant>>,
}

impl WindowManager {
    pub fn new() -> Result<Self, tauri::Error> {
        // First instantiate tracking areas manager so that it can be used by the windows.
        let tracking_areas_manager_arc = Arc::new(Mutex::new(TrackingAreasManager::new()));
        TrackingAreasManager::start_event_listeners(&tracking_areas_manager_arc);

        // Instantiate app windows. If this fails, the app will not work.
        let main_window = MainWindow::new()?;
        main_window.set_macos_properties();

        let main_window_arc = Arc::new(Mutex::new(main_window));
        MainWindow::start_event_listeners(&main_window_arc);

        let widget_window = WidgetWindow::new()?;
        widget_window.set_macos_properties();

        let widget_window_arc = Arc::new(Mutex::new(widget_window));
        WidgetWindow::start_event_listeners(&widget_window_arc);

        let code_overlay_window = CodeOverlayWindow::new()?;
        code_overlay_window.set_macos_properties();

        let code_overlay_window_arc = Arc::new(Mutex::new(code_overlay_window));
        CodeOverlayWindow::start_event_listeners(&code_overlay_window_arc);

        let explain_window = ExplainWindow::new()?;
        explain_window.set_macos_properties();

        let explain_window_arc = Arc::new(Mutex::new(explain_window));
        ExplainWindow::start_event_listeners(&explain_window_arc);

        Ok(Self {
            app_handle: app_handle(),
            editor_windows: Arc::new(Mutex::new(HashMap::new())),
            focused_editor_window: Arc::new(Mutex::new(None)),
            focused_app_window: None,
            is_core_engine_active: CORE_ENGINE_ACTIVE_AT_STARTUP,
            temporarily_hide_until: Arc::new(Mutex::new(std::time::Instant::now())),
        })
    }

    pub fn editor_windows(&self) -> &Arc<Mutex<HashMap<Uid, EditorWindow>>> {
        &self.editor_windows
    }

    pub fn clear_editor_windows(&mut self, editor_name: &String) {
        let mut editor_windows = self.editor_windows.lock();
        editor_windows.retain(|_, editor_window| {
            if editor_window.editor_name() != editor_name {
                true
            } else {
                self.focused_editor_window.lock().take();
                false
            }
        });
    }

    pub fn focused_editor_window(&self) -> Option<Uid> {
        if let Some(focused_editor_window) = self.focused_editor_window.try_lock() {
            focused_editor_window.clone()
        } else {
            debug!("Parking_lot: Could not lock focused_editor_window of WindowManager.");
            None
        }
    }

    pub fn is_core_engine_active(&self) -> bool {
        self.is_core_engine_active
    }

    pub fn set_is_core_engine_active(&mut self, is_core_engine_active: bool) {
        self.is_core_engine_active = is_core_engine_active;
    }

    pub fn set_focused_editor_window(&mut self, editor_window_hash: Option<Uid>) {
        *self.focused_editor_window.lock() = editor_window_hash;
    }

    pub fn set_focused_app_window(&mut self, app_window: AppWindow) {
        self.focused_app_window = Some(app_window);
    }

    pub fn hide_app_windows(&mut self, app_windows: Vec<AppWindow>) {
        if let Some(app_window) = self.focused_app_window.as_ref() {
            if app_windows.contains(app_window) {
                self.focused_app_window = None;
            }
        }

        EventWindowControls::AppWindowHide(HideAppWindowMessage {
            app_windows: app_windows.clone(),
        })
        .publish_to_tauri(&app_handle());
    }

    pub fn show_app_windows(
        &self,
        app_windows: Vec<AppWindow>,
        editor_id: Option<Uid>,
        explain_window_anchor: Option<LogicalFrame>,
    ) -> Option<()> {
        // If no editor id is given, the app windows are being shown in relation to the currently
        // focused editor window.
        let editor_id = if let Some(id) = editor_id {
            id
        } else {
            self.focused_editor_window()?
        };

        let editor_windows = self.editor_windows.lock();
        let editor_window = editor_windows.get(&editor_id)?;

        let textarea_position = editor_window.textarea_position(true)?;
        let textarea_size = editor_window.textarea_size()?;

        // double check that no windows are included that should not be shown
        // when the CoreEngine is not running
        let mut corrected_app_window_list = app_windows.clone();
        if !self.is_core_engine_active() {
            corrected_app_window_list.retain(|app_window| {
                if AppWindow::hidden_on_core_engine_inactive().contains(&app_window) {
                    false
                } else {
                    true
                }
            });
        }

        EventWindowControls::AppWindowShow(ShowAppWindowMessage {
            app_windows: corrected_app_window_list,
            editor_textarea: LogicalFrame {
                origin: textarea_position,
                size: textarea_size,
            },
            widget_position: editor_window.widget_position(true),
            monitor: editor_window.get_monitor(&self.app_handle)?,
            explain_window_anchor,
            viewport: editor_window.viewport()?,
            code_document: editor_window.code_document()?,
        })
        .publish_to_tauri(&app_handle());

        Some(())
    }

    pub fn temporarily_hide_app_windows(&mut self, app_windows: Vec<AppWindow>) {
        self.hide_app_windows(app_windows.clone());

        // Check if another instance of this routine is already running
        // Update the Instant time stamp when the widget should be shown again
        if *self.temporarily_hide_until.lock() > Instant::now() {
            *self.temporarily_hide_until.lock() =
                Instant::now() + Duration::from_millis(HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS);

            return;
        } else {
            *self.temporarily_hide_until.lock() =
                Instant::now() + Duration::from_millis(HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS);
        }

        let app_windows_shown;
        if !self.is_core_engine_active && app_windows.contains(&AppWindow::Widget) {
            app_windows_shown = vec![AppWindow::Widget];
        } else {
            app_windows_shown = AppWindow::shown_on_focus_gained(Some(app_windows.clone()));
        }

        std::thread::spawn({
            let editor_windows = self.editor_windows.clone();
            let focused_editor_window = self.focused_editor_window.clone();
            let temporarily_hide = self.temporarily_hide_until.clone();
            move || {
                loop {
                    let hide_until;
                    {
                        hide_until = temporarily_hide.lock().clone();
                    }

                    // Is zero when hide_until is older than Instant::now()
                    let duration = hide_until.duration_since(Instant::now());

                    if duration.is_zero() {
                        Self::delayed_show_app_windows(
                            &editor_windows,
                            &focused_editor_window,
                            app_windows_shown,
                        );
                        break;
                    }

                    std::thread::sleep(duration);
                }
            }
        });
    }

    pub fn delayed_show_app_windows(
        editor_windows_arc: &Arc<Mutex<HashMap<Uid, EditorWindow>>>,
        focused_editor_window_arc: &Arc<Mutex<Option<Uid>>>,
        app_windows_shown: Vec<AppWindow>,
    ) -> Option<()> {
        // Attempt try_lock() and early return if this fails; prevents deadlocks from happening.
        let editor_windows = editor_windows_arc.try_lock()?;

        let focused_editor_window = focused_editor_window_arc.lock();

        let editor_uid = focused_editor_window.as_ref()?;
        let editor_window = editor_windows.get(editor_uid)?;

        let textarea_position = editor_window.textarea_position(true)?;
        let textarea_size = editor_window.textarea_size()?;

        EventWindowControls::AppWindowShow(ShowAppWindowMessage {
            app_windows: app_windows_shown,
            editor_textarea: LogicalFrame {
                origin: textarea_position,
                size: textarea_size,
            },
            widget_position: editor_window.widget_position(true),
            monitor: editor_window.get_monitor(&app_handle())?,
            explain_window_anchor: None,
            viewport: editor_window.viewport()?,
            code_document: editor_window.code_document()?,
        })
        .publish_to_tauri(&app_handle());

        Some(())
    }

    pub fn update_app_windows(
        &self,
        app_windows: Vec<AppWindow>,
        update_editor_props: Option<ViewportPropertiesUpdateMessage>,
        update_window_position: Option<LogicalPosition>,
        update_window_size: Option<LogicalSize>,
    ) -> Option<()> {
        if let Some(update_editor_props) = update_editor_props.clone() {
            {
                let mut editor_windows = self.editor_windows.try_lock()?;
                let editor_window =
                    editor_windows.get_mut(&update_editor_props.viewport_properties.window_uid)?;

                editor_window.set_viewport(Some(update_editor_props.viewport_properties.clone()));
                editor_window.set_code_document(Some(
                    update_editor_props.code_document_frame_properties.clone(),
                ));
            }
        }

        EventWindowControls::AppWindowUpdate(UpdateAppWindowMessage {
            app_windows,
            viewport: update_editor_props
                .clone()
                .map(|props| props.viewport_properties),
            code_document: update_editor_props
                .clone()
                .map(|props| props.code_document_frame_properties),
            window_position: update_window_position,
            window_size: update_window_size,
        })
        .publish_to_tauri(&app_handle());

        Some(())
    }

    pub fn start_event_listeners(window_manager: &Arc<Mutex<WindowManager>>) {
        app_listener(window_manager);
        user_interaction_listener(window_manager);
        xcode_listener(window_manager);
        rule_execution_listener(window_manager);
        viewport_update_listener(window_manager);
    }
}

#[tauri::command]
pub fn cmd_resize_window(app_window: AppWindow, size_x: u32, size_y: u32) {
    EventWindowControls::AppWindowUpdate(UpdateAppWindowMessage {
        app_windows: vec![app_window],
        viewport: None,
        code_document: None,
        window_position: None,
        window_size: Some(LogicalSize {
            width: size_x as f64,
            height: size_y as f64,
        }),
    })
    .publish_to_tauri(&app_handle());
}
