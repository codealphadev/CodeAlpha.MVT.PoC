use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::HideAppWindowMessage, windows::SettingsWindow,
};

pub fn on_hide_app_window(
    settings_window: &Arc<Mutex<SettingsWindow>>,
    hide_msg: &HideAppWindowMessage,
) -> Option<()> {
    if hide_msg.app_windows.contains(&AppWindow::Settings) {
        let settings_window = settings_window.lock();

        if settings_window.hide().is_none() {
            println!("Failed to hide settings window");
        };
    }

    Some(())
}
