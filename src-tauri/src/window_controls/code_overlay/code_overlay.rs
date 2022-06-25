use tauri::{Error, Manager};

use cocoa::{base::id, foundation::NSInteger};
use objc::{class, msg_send, sel, sel_impl};

use crate::{
    core_engine::MatchRectangle,
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls::{
        actions::{close_window, open_window, resize_window, set_position},
        config::AppWindow,
    },
};

/// It opens the code overlay window and sets its position and size to match the textarea
///
/// Arguments:
///
/// * `app_handle`: The handle to the application.
///
/// Returns:
///
/// A Result<(), Error>
pub fn show_code_overlay(
    app_handle: &tauri::AppHandle,
    textarea_position: Option<tauri::LogicalPosition<f64>>,
    textarea_size: Option<tauri::LogicalSize<f64>>,
) -> Result<(), Error> {
    if let (Some(origin), Some(size)) = (textarea_position, textarea_size) {
        resize_window(app_handle, AppWindow::CodeOverlay, &size)?;
        set_position(app_handle, AppWindow::CodeOverlay, &origin)?;

        open_window(app_handle, AppWindow::CodeOverlay);

        app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            "event-compute-height",
            {
                MatchRectangle {
                    origin: LogicalPosition::from_tauri_LogicalPosition(&origin),
                    size: LogicalSize::from_tauri_LogicalSize(&size),
                }
            },
        )?;
    }

    if is_main_thread().unwrap() {
        configure_code_overlay_properties(app_handle);
    }

    Ok(())
}

/// It closes the code overlay window
///
/// Arguments:
///
/// * `app_handle`: The handle to the tauri app.
pub fn hide_code_overlay(app_handle: &tauri::AppHandle) {
    close_window(app_handle, AppWindow::CodeOverlay);
}

/// It gets the window handle for the code overlay window, and then sets the `ignoresMouseEvents`
/// property to `true`
///
/// Arguments:
///
/// * `app_handle`: The app handle that you can get from the tauri::AppBuilder.
fn configure_code_overlay_properties(app_handle: &tauri::AppHandle) {
    if let (Some(overlay_window), Some(widget_window)) = (
        app_handle.get_window(&AppWindow::CodeOverlay.to_string()),
        app_handle.get_window(&AppWindow::Widget.to_string()),
    ) {
        if let (Ok(ns_window_ptr_overlay), Ok(ns_window_ptr_widget)) =
            (overlay_window.ns_window(), widget_window.ns_window())
        {
            // Setting the mouse events to be ignored for the overlay window.
            unsafe {
                if !msg_send![ns_window_ptr_overlay as id, ignoresMouseEvents] {
                    let _: () = msg_send![ns_window_ptr_overlay as id, setIgnoresMouseEvents: true];
                }
            }

            // Ordering the widget window to the front. This prevents overlap.
            unsafe {
                let overlay_window_level: i64 = msg_send![ns_window_ptr_overlay as id, level];

                let _: () = msg_send![
                    ns_window_ptr_widget as id,
                    setLevel: overlay_window_level + 1 as NSInteger
                ];
            }
        }
    }
}

fn is_main_thread() -> Option<bool> {
    unsafe { Some(msg_send![class!(NSThread), isMainThread]) }
}
