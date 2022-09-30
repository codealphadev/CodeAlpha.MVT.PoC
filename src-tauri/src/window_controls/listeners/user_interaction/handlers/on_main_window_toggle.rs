use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{config::AppWindow, WindowManager};

pub fn on_main_window_toggle(
    window_manager: &Arc<Mutex<WindowManager>>,
    should_open_main_window: bool,
) -> Option<()> {
    let mut window_manager = window_manager.lock();

    // Depending on the activation status, we either show or hide the CodeOverlay window.
    if should_open_main_window {
        window_manager.show_app_windows(AppWindow::shown_on_click_widget(), None, None)?
    } else {
        window_manager.hide_app_windows(AppWindow::hidden_on_click_widget());
    }

    Some(())
}
