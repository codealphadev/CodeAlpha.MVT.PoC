use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::HideAppWindowMessage, windows::WidgetWindow,
};

pub fn on_hide_app_window(
    widget_window: &Arc<Mutex<WidgetWindow>>,
    hide_msg: &HideAppWindowMessage,
) -> Option<()> {
    if hide_msg.app_windows.contains(&AppWindow::Widget) {
        let widget_window = widget_window.lock();

        if widget_window.hide().is_none() {
            println!("Failed to hide widget window");
        };
    }

    if hide_msg.app_windows.contains(&AppWindow::Main) {
        let mut widget_window = widget_window.lock();

        widget_window.set_main_window_shown(false);
    }

    Some(())
}
