use std::sync::Arc;

use parking_lot::Mutex;

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
            println!("Failed to show widget window");
        }
    }

    Some(())
}
