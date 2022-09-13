use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::HideAppWindowMessage, windows::ExplainWindow,
};

pub fn on_hide_app_window(
    explain_window: &Arc<Mutex<ExplainWindow>>,
    hide_msg: &HideAppWindowMessage,
) -> Option<()> {
    if hide_msg.app_windows.contains(&AppWindow::Explain) {
        let mut explain_window = explain_window.try_lock()?;

        if explain_window.hide().is_none() {
            println!("Failed to hide explain window");
        };
    }

    Some(())
}
