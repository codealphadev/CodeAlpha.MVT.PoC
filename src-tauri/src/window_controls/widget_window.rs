use std::sync::{Arc, Mutex};

use tauri::{Error, Manager};
use tokio::time::{sleep, Duration};

use crate::ax_interaction::{
    app_widget::observer_app,
    models::{EditorUIElementFocusedMessage, EditorWindowMovedMessage, EditorWindowResizedMessage},
    AXEventXcode, AX_EVENT_XCODE_CHANNEL,
};

use super::{create_window, editor_window::EditorWindow, AppWindow};

pub struct WidgetWindow {
    handle: tauri::AppHandle,
    open_editor_windows: Arc<Mutex<Vec<EditorWindow>>>,
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

        // Register listener for xcode events relevant for displaying/hiding and positioning the widget
        let editor_windows_move_copy = editor_windows.clone();
        app_handle.listen_global(AX_EVENT_XCODE_CHANNEL, move |msg| {
            let axevent_xcode: AXEventXcode =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_xcode {
                AXEventXcode::EditorWindowResized(msg) => {
                    Self::on_resize_editor_window(&editor_windows_move_copy, &msg);
                }
                AXEventXcode::EditorWindowMoved(msg) => {
                    Self::on_move_editor_window(&editor_windows_move_copy, &msg);
                }
                AXEventXcode::EditorUIElementFocused(msg) => {
                    Self::on_ui_element_focus_change(&editor_windows_move_copy, &msg);
                }
                AXEventXcode::EditorAppActivated(_) => {}
                AXEventXcode::EditorAppDeactivated(_) => {}
                _ => {}
            }
        });

        Ok(Self {
            handle: app_handle.clone(),
            open_editor_windows: editor_windows.clone(),
        })
    }

    fn on_resize_editor_window(
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

                tauri::async_runtime::spawn(async move {
                    sleep(Duration::from_millis(2000)).await;
                    println!("Resize");
                });

                break;
            }
        }
    }

    fn on_move_editor_window(
        editor_window_list: &Arc<Mutex<Vec<EditorWindow>>>,
        moved_msg: &EditorWindowMovedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        for window in &mut *editor_list_locked {
            if window.id == moved_msg.id {
                window.update_window_dimensions(moved_msg.window_position, moved_msg.window_size);

                break;
            }
        }
    }

    fn on_ui_element_focus_change(
        editor_window_list: &Arc<Mutex<Vec<EditorWindow>>>,
        focus_msg: &EditorUIElementFocusedMessage,
    ) {
        let mut editor_list_locked = editor_window_list.lock().unwrap();

        for window in &mut *editor_list_locked {
            if window.id == focus_msg.window_id {
                window.update_focused_ui_element(
                    &focus_msg.focused_ui_element,
                    focus_msg.textarea_position,
                    focus_msg.textarea_size,
                );

                println!("UI ELEMENT");

                break;
            }
        }
    }
}
