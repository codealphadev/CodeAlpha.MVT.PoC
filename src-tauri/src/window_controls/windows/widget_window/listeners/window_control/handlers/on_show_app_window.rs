use std::sync::Arc;

use parking_lot::Mutex;
use tracing::warn;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::ShowAppWindowMessage, windows::WidgetWindow,
};

pub fn on_show_app_window(
    widget_window: &Arc<Mutex<WidgetWindow>>,
    show_msg: &ShowAppWindowMessage,
) -> Option<()> {
    if show_msg.app_windows.contains(&AppWindow::Widget) {
        let widget_window = widget_window.lock();

        if widget_window
            .show(
                &show_msg.widget_position,
                &show_msg.editor_textarea,
                &show_msg.monitor,
            )
            .is_none()
        {
            warn!("Failed to show widget window");
        }
    }

    if show_msg.app_windows.contains(&AppWindow::Main) {
        let mut widget_window = widget_window.lock();

        widget_window.set_main_window_shown(Some(true));
    }

    Some(())
}
