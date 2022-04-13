use std::sync::{Arc, Mutex};

use tauri::{Error, Manager};
use tokio::time::{sleep, Duration, Instant, Sleep};

use crate::{
    ax_interaction::{
        app_widget::observer_app,
        models::{
            EditorUIElementFocusedMessage, EditorWindowMovedMessage, EditorWindowResizedMessage,
        },
        AXEventXcode, AX_EVENT_XCODE_CHANNEL,
    },
    window_controls::{close_window, open_window},
};

use super::{create_window, editor_window::EditorWindow, AppWindow};

static HIDE_DELAY_IN_MILLIS: u64 = 200;

#[allow(dead_code)]
pub struct WidgetWindow {
    handle: tauri::AppHandle,
    open_editor_windows: Arc<Mutex<Vec<EditorWindow>>>,
    temporarily_hide_timer: Arc<tokio::sync::Mutex<Option<Sleep>>>,
    currently_focused_editor_window: Arc<Mutex<Option<uuid::Uuid>>>,
}

impl WidgetWindow {
    pub fn new(
        app_handle: &tauri::AppHandle,
        editor_windows: &Arc<Mutex<Vec<EditorWindow>>>,
    ) -> Result<Self, Error> {
        // Create Tauri Window
        create_window(&app_handle, AppWindow::Widget)?;

        // Register Observer for Widget AX Events
        if observer_app(&app_handle).is_err() {
            return Err(Error::CreateWindow);
        }

        // Instantiate a timer shared between closure calls for incoming events
        // Each qualifying incoming event hides the widget and resets the timer for when it should be shown again
        let temporarily_hide_timer: Arc<tokio::sync::Mutex<Option<Sleep>>> =
            Arc::new(tokio::sync::Mutex::new(None));

        let currently_focused_editor_window: Arc<Mutex<Option<uuid::Uuid>>> =
            Arc::new(Mutex::new(None));

        // Spin up watcher for when to show/hide the widget
        Self::control_widget_visibility(
            &currently_focused_editor_window,
            &temporarily_hide_timer,
            app_handle,
        );

        // Register listener for xcode events relevant for displaying/hiding and positioning the widget
        let editor_windows_move_copy = editor_windows.clone();
        let hide_timer_move_copy = temporarily_hide_timer.clone();
        let focused_editor_move_copy = currently_focused_editor_window.clone();
        app_handle.listen_global(AX_EVENT_XCODE_CHANNEL, move |msg| {
            let axevent_xcode: AXEventXcode =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_xcode {
                AXEventXcode::EditorWindowResized(msg) => {
                    Self::on_resize_editor_window(
                        &hide_timer_move_copy,
                        &editor_windows_move_copy,
                        &msg,
                    );
                }
                AXEventXcode::EditorWindowMoved(msg) => {
                    Self::on_move_editor_window(
                        &hide_timer_move_copy,
                        &editor_windows_move_copy,
                        &msg,
                    );
                }
                AXEventXcode::EditorUIElementFocused(msg) => {
                    Self::on_ui_element_focus_change(
                        &focused_editor_move_copy,
                        &editor_windows_move_copy,
                        &msg,
                    );
                }
                AXEventXcode::EditorAppActivated(_) => {}
                AXEventXcode::EditorAppDeactivated(_) => {}
                _ => {}
            }
        });

        Ok(Self {
            handle: app_handle.clone(),
            open_editor_windows: editor_windows.clone(),
            temporarily_hide_timer,
            currently_focused_editor_window,
        })
    }

    fn on_resize_editor_window(
        hide_timer: &Arc<tokio::sync::Mutex<Option<Sleep>>>,
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
                let hide_timer_move_copy = hide_timer.clone();
                tauri::async_runtime::spawn(async move {
                    let mut timer_locked = hide_timer_move_copy.lock().await;

                    *timer_locked = Some(tokio::time::sleep(Duration::from_millis(
                        HIDE_DELAY_IN_MILLIS,
                    )));
                });

                break;
            }
        }
    }

    fn on_move_editor_window(
        hide_timer: &Arc<tokio::sync::Mutex<Option<Sleep>>>,
        editor_window_list: &Arc<Mutex<Vec<EditorWindow>>>,
        moved_msg: &EditorWindowMovedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        for window in &mut *editor_list_locked {
            if window.id == moved_msg.id {
                window.update_window_dimensions(moved_msg.window_position, moved_msg.window_size);

                // Reset hide timer after which the widget should be displayed again
                let hide_timer_move_copy = hide_timer.clone();
                tauri::async_runtime::spawn(async move {
                    let mut timer_locked = hide_timer_move_copy.lock().await;

                    *timer_locked = Some(tokio::time::sleep(Duration::from_millis(
                        HIDE_DELAY_IN_MILLIS,
                    )));
                });

                break;
            }
        }
    }

    /// Update EditorWindow to which of it's ui elements is currently in focus. Furthermore, also update
    /// which of all open editor windows is currently in focus.
    fn on_ui_element_focus_change(
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

    fn control_widget_visibility(
        focused_editor: &Arc<Mutex<Option<uuid::Uuid>>>,
        temporarily_hide_timer: &Arc<tokio::sync::Mutex<Option<tokio::time::Sleep>>>,
        app_handle: &tauri::AppHandle,
    ) {
        let hide_timer_move_copy = temporarily_hide_timer.clone();
        let app_handle_move_copy = app_handle.clone();
        let focused_editor_move_copy = focused_editor.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                let timer_locked = hide_timer_move_copy.lock().await;

                if let Some(timer) = &*timer_locked {
                    let foo = timer.deadline().duration_since(Instant::now());

                    if foo.as_millis() > 0 {
                        close_window(&app_handle_move_copy, AppWindow::Widget);
                    } else {
                        open_window(&app_handle_move_copy, AppWindow::Widget);
                    }
                }

                sleep(Duration::from_millis(25)).await;
            }
        });
    }
}
