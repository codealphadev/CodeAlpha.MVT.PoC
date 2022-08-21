use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::events::models::CoreActivationStatusMessage,
    window_controls::{config::AppWindow, WindowManager},
};

pub fn on_core_activation_status_update(
    window_manager: &Arc<Mutex<WindowManager>>,
    activation_msg: &CoreActivationStatusMessage,
) -> Option<()> {
    let mut window_manager = window_manager.lock();

    window_manager.set_is_core_engine_active(activation_msg.engine_active);

    // Depending on the activation status, we either show or hide the CodeOverlay window.
    if activation_msg.engine_active {
        window_manager.show_app_windows(AppWindow::shown_on_core_engine_activated(), None)?
    } else {
        window_manager.hide_app_windows(AppWindow::hidden_on_core_engine_inactive());
    }

    Some(())
}
