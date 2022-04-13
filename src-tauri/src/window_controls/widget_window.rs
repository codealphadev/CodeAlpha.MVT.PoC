use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use tauri::{Error, Manager};

use crate::{
    ax_interaction::{
        app::observer_app::register_observer_app,
        models::editor::{
            EditorAppActivatedMessage, EditorAppDeactivatedMessage, EditorUIElementFocusedMessage,
            EditorWindowMovedMessage, EditorWindowResizedMessage,
        },
        AXEventApp, AXEventXcode, AX_EVENT_APP_CHANNEL, AX_EVENT_XCODE_CHANNEL,
    },
    window_controls::{close_window, open_window},
};

use super::{create_window, editor_window::EditorWindow, AppWindow};

static HIDE_DELAY_IN_MILLIS: u64 = 200;

#[allow(dead_code)]
pub struct WidgetWindow {
    handle: tauri::AppHandle,

    // List of open editor windows. List is managed by WindowStateManager.
    editor_windows: Arc<Mutex<Vec<EditorWindow>>>,

    // Identitfier of the currently focused editor window. Is None until the first window was focused.
    currently_focused_editor_window: Arc<Mutex<Option<uuid::Uuid>>>,

    // Each qualifying incoming event updates the instant until when the widget should be hidden.
    hide_until_instant: Arc<Mutex<Instant>>,

    // Boolean saying if the currently focused application is Xcode.
    is_xcode_focused: Arc<Mutex<Option<bool>>>,

    // Boolean saying if the currently focused application is our app.
    is_app_focused: Arc<Mutex<Option<bool>>>,

    // Identitfier of the currently focused app window. Is None until the first window was focused.
    currently_focused_app_window: Arc<Mutex<Option<AppWindow>>>,
}

impl WidgetWindow {
    pub fn new(
        app_handle: &tauri::AppHandle,
        editor_windows: &Arc<Mutex<Vec<EditorWindow>>>,
    ) -> Result<Self, Error> {
        // Create Tauri Window
        create_window(&app_handle, AppWindow::Widget)?;

        // Register Observer for Widget AX Events
        if register_observer_app(&app_handle).is_err() {
            return Err(Error::CreateWindow);
        }

        // Instantiate an instant shared between closure calls for incoming events
        let hide_until_instant: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));

        let currently_focused_editor_window: Arc<Mutex<Option<uuid::Uuid>>> =
            Arc::new(Mutex::new(None));
        let currently_focused_app_window: Arc<Mutex<Option<AppWindow>>> =
            Arc::new(Mutex::new(None));

        let is_xcode_focused: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
        let is_app_focused: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));

        // Spin up watcher for when to show/hide the widget
        // Self::control_widget_visibility(
        //     &is_xcode_focused,
        //     &currently_focused_editor_window,
        //     &hide_until_instant,
        //     app_handle,
        // );

        // Register listener for AXEvents from our app
        Self::register_listener_app(&app_handle, &currently_focused_app_window, &is_app_focused);

        // Register listener for xcode events relevant for displaying/hiding and positioning the widget
        Self::register_listener_xcode(
            &app_handle,
            &editor_windows,
            &currently_focused_editor_window,
            &hide_until_instant,
            &is_xcode_focused,
        );

        Ok(Self {
            handle: app_handle.clone(),
            editor_windows: editor_windows.clone(),
            hide_until_instant,
            currently_focused_editor_window,
            is_xcode_focused,
            is_app_focused,
            currently_focused_app_window,
        })
    }

    fn register_listener_xcode(
        app_handle: &tauri::AppHandle,
        editor_windows: &Arc<Mutex<Vec<EditorWindow>>>,
        currently_focused_editor_window: &Arc<Mutex<Option<uuid::Uuid>>>,
        hide_until_instant: &Arc<Mutex<Instant>>,
        is_xcode_focused: &Arc<Mutex<Option<bool>>>,
    ) {
        let editor_windows_move_copy = editor_windows.clone();
        let hide_until_instant_move_copy = hide_until_instant.clone();
        let focused_editor_move_copy = currently_focused_editor_window.clone();
        let is_xcode_focused_move_copy = is_xcode_focused.clone();
        app_handle.listen_global(AX_EVENT_XCODE_CHANNEL, move |msg| {
            let axevent_xcode: AXEventXcode =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_xcode {
                AXEventXcode::EditorWindowResized(msg) => {
                    Self::on_resize_editor_window(
                        &hide_until_instant_move_copy,
                        &editor_windows_move_copy,
                        &msg,
                    );
                }
                AXEventXcode::EditorWindowMoved(msg) => {
                    Self::on_move_editor_window(
                        &hide_until_instant_move_copy,
                        &editor_windows_move_copy,
                        &msg,
                    );
                }
                AXEventXcode::EditorUIElementFocused(msg) => {
                    Self::on_editor_ui_element_focus_change(
                        &focused_editor_move_copy,
                        &editor_windows_move_copy,
                        &msg,
                    );
                }
                AXEventXcode::EditorAppActivated(msg) => {
                    Self::on_editor_app_activated(&is_xcode_focused_move_copy, &msg)
                }
                AXEventXcode::EditorAppDeactivated(msg) => {
                    Self::on_editor_app_deactivated(&is_xcode_focused_move_copy, &msg)
                }
                _ => {}
            }
        });
    }

    fn register_listener_app(
        app_handle: &tauri::AppHandle,
        currently_focused_app_window: &Arc<Mutex<Option<AppWindow>>>,
        is_app_focused: &Arc<Mutex<Option<bool>>>,
    ) {
        let is_app_focused_move_copy = is_app_focused.clone();
        let focused_app_window_move_copy = currently_focused_app_window.clone();
        app_handle.listen_global(AX_EVENT_APP_CHANNEL, move |msg| {
            let axevent_app: AXEventApp = serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match &axevent_app {
                AXEventApp::AppWindowFocused(msg) => {
                    let mut focused_app_window_locked =
                        focused_app_window_move_copy.lock().unwrap();

                    *focused_app_window_locked = Some(msg.window);
                }
                AXEventApp::AppWindowMoved(_) => {
                    // Recalculate boundaries
                }
                AXEventApp::AppUIElementFocused(_) => {
                    // TODO: Do nothing
                }
                AXEventApp::AppActivated(_) => {
                    let mut is_app_focused_locked = is_app_focused_move_copy.lock().unwrap();

                    *is_app_focused_locked = Some(true);
                }
                AXEventApp::AppDeactivated(_) => {
                    let mut is_app_focused_locked = is_app_focused_move_copy.lock().unwrap();

                    *is_app_focused_locked = Some(false);
                }
                AXEventApp::None => {}
            }

            println!("{:#?}", axevent_app);
        });
    }

    fn on_resize_editor_window(
        hide_until_instant: &Arc<Mutex<Instant>>,
        editor_window_list: &Arc<Mutex<Vec<EditorWindow>>>,
        resize_msg: &EditorWindowResizedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        for window in &mut *editor_list_locked {
            if window.id == resize_msg.id {
                window.update_window_dimensions(resize_msg.window_position, resize_msg.window_size);

                if let (Some(position), Some(size)) =
                    (resize_msg.textarea_position, resize_msg.textarea_size)
                {
                    window.update_textarea_dimensions(position, size);
                }

                // Reset hide timer after which the widget should be displayed again
                let mut hide_until_instant_locked = hide_until_instant.lock().unwrap();
                *hide_until_instant_locked =
                    Instant::now() + Duration::from_millis(HIDE_DELAY_IN_MILLIS);

                break;
            }
        }
    }

    fn on_move_editor_window(
        hide_until_instant: &Arc<Mutex<Instant>>,
        editor_window_list: &Arc<Mutex<Vec<EditorWindow>>>,
        moved_msg: &EditorWindowMovedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        for window in &mut *editor_list_locked {
            if window.id == moved_msg.id {
                window.update_window_dimensions(moved_msg.window_position, moved_msg.window_size);

                // Reset hide timer after which the widget should be displayed again
                let mut hide_until_instant_locked = hide_until_instant.lock().unwrap();
                *hide_until_instant_locked =
                    Instant::now() + Duration::from_millis(HIDE_DELAY_IN_MILLIS);

                break;
            }
        }
    }

    /// Update EditorWindow to which of it's ui elements is currently in focus. Furthermore, also update
    /// which of all open editor windows is currently in focus.
    fn on_editor_ui_element_focus_change(
        focused_editor: &Arc<Mutex<Option<uuid::Uuid>>>,
        editor_window_list: &Arc<Mutex<Vec<EditorWindow>>>,
        focus_msg: &EditorUIElementFocusedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();
        let mut focused_editor_locked = focused_editor.lock().unwrap();

        for window in &mut *editor_list_locked {
            if window.id == focus_msg.window_id {
                window.update_focused_ui_element(
                    &focus_msg.focused_ui_element,
                    focus_msg.textarea_position,
                    focus_msg.textarea_size,
                );

                // Set which editor window is currently focused
                *focused_editor_locked = Some(window.id);

                break;
            }
        }
    }

    fn on_editor_app_deactivated(
        is_xcode_focused: &Arc<Mutex<Option<bool>>>,
        deactivated_msg: &EditorAppDeactivatedMessage,
    ) {
        let mut is_xcode_focused_locked = is_xcode_focused.lock().unwrap();

        if deactivated_msg.editor_name == "Xcode" {
            *is_xcode_focused_locked = Some(false);
        }
    }

    fn on_editor_app_activated(
        is_xcode_focused: &Arc<Mutex<Option<bool>>>,
        activated_msg: &EditorAppActivatedMessage,
    ) {
        let mut is_xcode_focused_locked = is_xcode_focused.lock().unwrap();

        if activated_msg.editor_name == "Xcode" {
            *is_xcode_focused_locked = Some(true);
        }
    }

    fn _control_widget_visibility(
        is_xcode_focused: &Arc<Mutex<Option<bool>>>,
        focused_editor: &Arc<Mutex<Option<uuid::Uuid>>>,
        hide_until_instant: &Arc<Mutex<Instant>>,
        app_handle: &tauri::AppHandle,
    ) {
        let hide_until_instant_move_copy = hide_until_instant.clone();
        let app_handle_move_copy = app_handle.clone();
        let _focused_editor_move_copy = focused_editor.clone();
        let is_xcode_focused_move_copy = is_xcode_focused.clone();
        thread::spawn(move || loop {
            // Sleep first to not block the locked Mutexes afterwards
            thread::sleep(std::time::Duration::from_millis(25));

            let hide_until_locked = hide_until_instant_move_copy.lock().unwrap();
            let is_xcode_focused_locked = is_xcode_focused_move_copy.lock().unwrap();

            // Hide widget if editor is not in focus
            if (*is_xcode_focused_locked).is_none() {
                close_window(&app_handle_move_copy, AppWindow::Widget);
                continue;
            }

            if let Some(xcode_focused) = *is_xcode_focused_locked {
                if xcode_focused {
                    if *hide_until_locked > Instant::now() {
                        close_window(&app_handle_move_copy, AppWindow::Widget);
                    } else {
                        open_window(&app_handle_move_copy, AppWindow::Widget);
                    }
                } else {
                    close_window(&app_handle_move_copy, AppWindow::Widget);
                }
            }
        });
    }
}
