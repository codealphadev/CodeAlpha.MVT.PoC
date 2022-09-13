use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    utils::geometry::LogicalFrame,
    window_controls::{
        config::AppWindow, events::models::app_window::ShowAppWindowMessage, windows::ExplainWindow,
    },
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
                &LogicalFrame {
                    origin: show_msg.editor_textarea.origin.to_owned(),
                    size: show_msg.editor_textarea.size.to_owned(),
                },
                &show_msg.monitor,
            )
            .is_none()
        {
            println!("Failed to show explain window");
        };
    }

    Some(())
}
