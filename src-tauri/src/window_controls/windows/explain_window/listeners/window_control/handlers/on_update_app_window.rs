use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{
    config::AppWindow, models::app_window::UpdateAppWindowMessage, windows::ExplainWindow,
};

pub fn on_update_app_window(
    explain_window: &Arc<Mutex<ExplainWindow>>,
    update_msg: &UpdateAppWindowMessage,
) -> Option<()> {
    if update_msg.app_windows.contains(&AppWindow::Explain) {
        let mut explain_window = explain_window.try_lock()?;

        explain_window.update(
            &update_msg.viewport,
            &update_msg.code_document,
            &update_msg.window_position,
            &update_msg.window_size,
        );
    }

    Some(())
}
