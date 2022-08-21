use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::ShowAppWindowMessage, windows::CodeOverlayWindow,
};

pub fn on_show_app_window(
    code_overlay_window: &Arc<Mutex<CodeOverlayWindow>>,
    show_msg: &ShowAppWindowMessage,
) -> Option<()> {
    if show_msg.app_windows.contains(&AppWindow::CodeOverlay) {
        let code_overlay_window = code_overlay_window.lock();

        if code_overlay_window
            .show(
                &show_msg.editor_textarea.origin,
                &show_msg.editor_textarea.size,
            )
            .is_none()
        {
            println!("Failed to show code overlay window");
        };
    }

    Some(())
}
