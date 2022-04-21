use cocoa::{appkit::NSWindowOrderingMode, base::id};
use objc::{msg_send, sel, sel_impl};
use tauri::{Error, Manager};

use crate::window_controls::{
    actions::{current_monitor_of_window, get_position, get_size, set_position},
    widget_window::{
        prevent_misalignement_of_content_and_widget, POSITIONING_OFFSET_X, POSITIONING_OFFSET_Y,
    },
    AppWindow,
};

#[derive(Clone, serde::Serialize)]
struct ContentWindowOrientationEvent {
    orientation_right: bool,
}

fn resize(
    app_handle: &tauri::AppHandle,
    updated_size: &tauri::LogicalSize<f64>,
) -> Result<(), Error> {
    if let Some(content_window) = app_handle.get_window(&AppWindow::Content.to_string()) {
        content_window.set_size(tauri::Size::Logical(*updated_size))
    } else {
        Err(Error::WebviewNotFound)
    }
}

fn reposition(app_handle: &tauri::AppHandle) -> Result<(), Error> {
    let widget_position = get_position(&app_handle, AppWindow::Widget)?;
    let widget_size = get_size(&app_handle, AppWindow::Widget)?;
    let content_position = get_position(&app_handle, AppWindow::Content)?;
    let content_size = get_size(&app_handle, AppWindow::Content)?;

    let mut new_content_pos = tauri::LogicalPosition {
        x: widget_position.x + (widget_size.width - content_size.width) + POSITIONING_OFFSET_X,
        y: widget_position.y - content_size.height - POSITIONING_OFFSET_Y,
    };

    // Check if content window orientation should be flipped in case it would go out of the left side of the screen
    let screen = current_monitor_of_window(&app_handle, AppWindow::Widget).unwrap();
    let screen_position = screen.position().to_logical::<f64>(screen.scale_factor());

    let mut bubble_orientation_right = true;
    if screen_position.x > new_content_pos.x {
        new_content_pos.x = widget_position.x - POSITIONING_OFFSET_X;
        bubble_orientation_right = false;
    }

    // Emit event to content window to update its orientation
    app_handle.emit_to(
        &AppWindow::Content.to_string(),
        "evt-content-window-orientation",
        ContentWindowOrientationEvent {
            orientation_right: bubble_orientation_right,
        },
    )?;

    if content_position != new_content_pos {
        set_position(&app_handle, AppWindow::Content, &new_content_pos)
    } else {
        Ok(())
    }
}

pub fn open(app_handle: &tauri::AppHandle) -> Result<(), Error> {
    // 1. Reposition widget in case it is moved too far up on the screen
    correct_widget_position(&app_handle);

    // 2. Position content window relative to widget position
    reposition(&app_handle)?;

    // 3. Set parent window
    // Here we need to go past the tauri APIs and use native macOS APIs to set the parent window at runtime.
    if let (Some(widget_window), Some(content_window)) = (
        app_handle.get_window(&AppWindow::Widget.to_string()),
        app_handle.get_window(&AppWindow::Content.to_string()),
    ) {
        if let (Ok(parent_ptr), Ok(child_ptr)) =
            (widget_window.ns_window(), content_window.ns_window())
        {
            unsafe {
                let _: () = msg_send![parent_ptr as id, addChildWindow: child_ptr as id ordered: NSWindowOrderingMode::NSWindowBelow];
            }
        }

        // 4. Show content
        content_window.show()
    } else {
        Err(Error::WebviewNotFound)
    }
}

pub fn hide(app_handle: &tauri::AppHandle) -> Result<(), Error> {
    // 1. Remove parent window
    // Here we need to go past the tauri APIs and use native macOS APIs to remove the parent window at runtime.
    if let (Some(widget_window), Some(content_window)) = (
        app_handle.get_window(&AppWindow::Widget.to_string()),
        app_handle.get_window(&AppWindow::Content.to_string()),
    ) {
        if let (Ok(parent_ptr), Ok(child_ptr)) =
            (widget_window.ns_window(), content_window.ns_window())
        {
            unsafe {
                let _: () = msg_send![parent_ptr as id, removeChildWindow: child_ptr as id];
            }
        }

        // 2. Hide content
        content_window.hide()
    } else {
        println!("hide: no widget or content window found");
        Err(Error::WebviewNotFound)
    }
}

pub fn is_open(app_handle: &tauri::AppHandle) -> Result<bool, Error> {
    if let Some(content_window) = app_handle.get_window(&AppWindow::Content.to_string()) {
        content_window.is_visible()
    } else {
        Err(Error::WebviewNotFound)
    }
}

#[tauri::command]
pub fn cmd_resize_content_window(app_handle: tauri::AppHandle, size_x: u32, size_y: u32) {
    let _ = resize(
        &app_handle,
        &tauri::LogicalSize {
            width: size_x as f64,
            height: size_y as f64,
        },
    );
}

#[tauri::command]
pub fn cmd_open_content_window(app_handle: tauri::AppHandle) {
    let _ = open(&app_handle);
}

#[tauri::command]
pub fn cmd_toggle_content_window(app_handle: tauri::AppHandle) {
    if let Ok(visible) = is_open(&app_handle) {
        if visible {
            let _ = hide(&app_handle);
        } else {
            let _ = open(&app_handle);
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
