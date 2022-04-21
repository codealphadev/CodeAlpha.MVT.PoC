use std::sync::{Arc, Mutex};

use cocoa::{appkit::NSWindowOrderingMode, base::id};
use objc::{msg_send, sel, sel_impl};
use tauri::{Error, Manager};

use crate::window_controls::{
    actions::{create_window, get_position, get_size, set_position},
    widget_window::{
        prevent_misalignement_of_content_and_widget, POSITIONING_OFFSET_X, POSITIONING_OFFSET_Y,
    },
    AppWindow,
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ContentWindow {
    app_handle: tauri::AppHandle,

    window: tauri::Window,
    size: Option<tauri::LogicalSize<f64>>,
    position: Option<tauri::LogicalPosition<f64>>,
}

impl ContentWindow {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let window = create_window(app_handle, AppWindow::Content).unwrap();

        Self {
            window,
            app_handle: app_handle.clone(),
            size: None,
            position: None,
        }
    }

    pub fn resize(&mut self, updated_size: &tauri::LogicalSize<f64>) {
        if self
            .window
            .set_size(tauri::Size::Logical(*updated_size))
            .is_ok()
        {
            self.size = Some(*updated_size);
        }
    }

    pub fn reposition(&mut self) {
        if let (Ok(widget_position), Ok(widget_size)) = (
            get_position(&self.app_handle, AppWindow::Widget),
            get_size(&self.app_handle, AppWindow::Widget),
        ) {
            if let Some(content_size) = self.size {
                let new_content_pos = tauri::LogicalPosition {
                    x: widget_position.x
                        + (widget_size.width - content_size.width)
                        + POSITIONING_OFFSET_X,
                    y: widget_position.y - content_size.height - POSITIONING_OFFSET_Y,
                };

                if set_position(&self.app_handle, AppWindow::Content, &new_content_pos).is_ok() {
                    self.position = Some(new_content_pos);
                }
            }
        }
    }

    pub fn open(&mut self) -> Result<(), Error> {
        // 1. Position relative to widget
        self.reposition();

        // 2. Set parent window
        // Here we need to go past the tauri APIs and use native macOS APIs to set the parent window at runtime.
        if let Some(parent_window) = self.app_handle.get_window(&AppWindow::Widget.to_string()) {
            if let (Ok(parent_ptr), Ok(child_ptr)) =
                (parent_window.ns_window(), self.window.ns_window())
            {
                unsafe {
                    let _: () = msg_send![parent_ptr as id, addChildWindow: child_ptr as id ordered: NSWindowOrderingMode::NSWindowBelow];
                }
            }
        }

        // 3. Open the window
        self.window.show()
    }

    pub fn hide(&mut self) -> Result<(), Error> {
        // 2. Remove parent window
        // Here we need to go past the tauri APIs and use native macOS APIs to remove the parent window at runtime.
        if let Some(parent_window) = self.app_handle.get_window(&AppWindow::Widget.to_string()) {
            if let (Ok(parent_ptr), Ok(child_ptr)) =
                (parent_window.ns_window(), self.window.ns_window())
            {
                unsafe {
                    let _: () = msg_send![parent_ptr as id, removeChildWindow: child_ptr as id];
                }
            }
        }

        self.window.hide()
    }

    pub fn is_open(&self) -> Result<bool, Error> {
        self.window.is_visible()
    }
}

#[tauri::command]
pub fn cmd_resize_content_window(
    size_x: u32,
    size_y: u32,
    state_content_window: tauri::State<Arc<Mutex<ContentWindow>>>,
) {
    let content_window = &mut *(state_content_window.lock().unwrap());

    content_window.resize(&tauri::LogicalSize {
        width: size_x as f64,
        height: size_y as f64,
    });
}

#[tauri::command]
pub fn cmd_open_content_window(
    app_handle: tauri::AppHandle,
    state_content_window: tauri::State<Arc<Mutex<ContentWindow>>>,
) {
    let content_window = &mut *(state_content_window.lock().unwrap());

    if content_window.open().is_ok() {
        // let activation_event =
        //     AXEventApp::AppContentActivationChange(AppContentActivationMessage {
        //         activation_state: ContentWindowState::Active,
        //     });

        // // Emit to rust listeners
        // activation_event.publish_to_tauri(&app_handle);
    }
}

#[tauri::command]
pub fn cmd_toggle_content_window(
    app_handle: tauri::AppHandle,
    state_content_window: tauri::State<Arc<Mutex<ContentWindow>>>,
) {
    let content_window = &mut *(state_content_window.lock().unwrap());

    if let Ok(visible) = content_window.is_open() {
        if visible {
            if content_window.hide().is_ok() {
                // let activation_event =
                //     AXEventApp::AppContentActivationChange(AppContentActivationMessage {
                //         activation_state: ContentWindowState::Inactive,
                //     });

                // // Emit to rust listeners
                // activation_event.publish_to_tauri(&app_handle);
            }
        } else {
            // Reposition widget in case it is moved too far up on the screen
            correct_widget_position(&app_handle);

            if content_window.open().is_ok() {
                // let activation_event =
                //     AXEventApp::AppContentActivationChange(AppContentActivationMessage {
                //         activation_state: ContentWindowState::Active,
                //     });

                // // Emit to rust listeners
                // activation_event.publish_to_tauri(&app_handle);
            }
        }
    } else {
        println!("Error: cmd_toggle_content_window");
    }
}

fn correct_widget_position(app_handle: &tauri::AppHandle) {
    if let Ok(widget_position) = get_position(app_handle, AppWindow::Widget) {
        let mut widget_position_updated = widget_position.clone();
        prevent_misalignement_of_content_and_widget(app_handle, &mut widget_position_updated);

        if widget_position != widget_position_updated {
            let _ = set_position(app_handle, AppWindow::Widget, &widget_position_updated);
        }
    }
}
