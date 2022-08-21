use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::HideAppWindowMessage, windows::CodeOverlayWindow,
};

pub fn on_hide_app_window(
    code_overlay_window: &Arc<Mutex<CodeOverlayWindow>>,
    hide_msg: &HideAppWindowMessage,
) -> Option<()> {
    if hide_msg.app_windows.contains(&AppWindow::CodeOverlay) {
        let code_overlay_window = code_overlay_window.lock();

        if code_overlay_window.hide().is_none() {
            println!("Failed to hide code overlay window");
        };
    }

    Some(())
}
