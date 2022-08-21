use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::{
        currently_focused_window, generate_axui_element_hash, models::app::AppDeactivatedMessage,
    },
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_deactivate_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    _deactivated_msg: &AppDeactivatedMessage,
) {
    let mut window_manager = window_manager.lock();

    window_manager.set_is_app_focused(false);

    // Determine if we need to hide our app
    // If the focus now is on a known editor window, we keep showing our app.
    // Subsequently arriving events will determine elsewhere if we need to hide our app.
    if let Ok(focused_window) = currently_focused_window() {
        let window_hash = generate_axui_element_hash(&focused_window);
        if window_manager
            .editor_windows()
            .lock()
            .get(&window_hash)
            .is_none()
        {
            window_manager.hide_app_windows(AppWindow::hidden_on_focus_lost())
        }
    } else {
        window_manager.hide_app_windows(AppWindow::hidden_on_focus_lost())
    }
}
