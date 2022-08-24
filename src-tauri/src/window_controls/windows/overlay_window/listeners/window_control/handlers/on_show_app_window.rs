use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::ShowAppWindowMessage, windows::OverlayWindow,
};

pub fn on_show_app_window(
    settings_window: &Arc<Mutex<OverlayWindow>>,
    show_msg: &ShowAppWindowMessage,
) -> Option<()> {
    if show_msg.app_windows.contains(&AppWindow::Overlay) {
        let settings_window = settings_window.lock();

        if settings_window
            .show(
                &show_msg.editor_textarea.origin,
                &show_msg.editor_textarea.size,
            )
            .is_none()
        {
            println!("Failed to show settings window");
        };
    }

    Some(())
}
