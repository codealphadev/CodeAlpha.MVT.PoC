use std::sync::Arc;

use parking_lot::Mutex;
use tracing::warn;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::ShowAppWindowMessage, windows::ExplainWindow,
};

pub fn on_show_app_window(
    explain_window: &Arc<Mutex<ExplainWindow>>,
    show_msg: &ShowAppWindowMessage,
) -> Option<()> {
    if show_msg.app_windows.contains(&AppWindow::Explain) {
        let mut explain_window = explain_window.try_lock()?;

        if explain_window
            .show(
                show_msg.explain_window_anchor,
                &show_msg.viewport,
                &show_msg.code_document,
                &show_msg.monitor,
            )
            .is_none()
        {
            warn!("Failed to show explain window");
        };
    }

    Some(())
}
