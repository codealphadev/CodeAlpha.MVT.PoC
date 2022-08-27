use std::sync::Arc;

use parking_lot::Mutex;

use crate::{platform::macos::models::app::AppActivatedMessage, window_controls::WindowManager};

pub fn on_activated_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    activated_msg: &AppActivatedMessage,
) {
    let mut window_manager = window_manager.lock();

    if let Some(focused_app_window) = activated_msg.focused_app_window {
        window_manager.set_focused_app_window(focused_app_window);
    }
}
