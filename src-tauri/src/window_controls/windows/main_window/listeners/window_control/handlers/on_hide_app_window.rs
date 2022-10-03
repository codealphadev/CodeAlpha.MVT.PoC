use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use cocoa::base::id;
use objc::{msg_send, sel, sel_impl};
use tracing::debug;

use crate::{
    app_handle,
    window_controls::{
        config::AppWindow, events::models::app_window::HideAppWindowMessage, windows::MainWindow,
    },
};

pub fn on_hide_app_window(
    main_window: &Arc<Mutex<MainWindow>>,
    hide_msg: &HideAppWindowMessage,
) -> Option<()> {
    if hide_msg.app_windows.contains(&AppWindow::Main) {
        let main_window = main_window.lock();

        if main_window.hide().is_none() {
            debug!("Failed to hide MainWindow (on_hide_app_window.rs)");
        };

        // Restore parent child relationship
        let widget_tauri_window = app_handle().get_window(&AppWindow::Widget.to_string())?;
        let main_tauri_window = app_handle().get_window(&AppWindow::Main.to_string())?;
        if let (Ok(parent_ptr), Ok(child_ptr)) = (
            widget_tauri_window.ns_window(),
            main_tauri_window.ns_window(),
        ) {
            unsafe {
                let _: () = msg_send![parent_ptr as id, removeChildWindow: child_ptr as id];
            }
        }
    }

    Some(())
}
