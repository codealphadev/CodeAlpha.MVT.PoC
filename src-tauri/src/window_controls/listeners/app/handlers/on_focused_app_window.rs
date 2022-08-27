use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::app::AppWindowFocusedMessage, window_controls::WindowManager,
};

pub fn on_focused_app_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    focused_msg: &AppWindowFocusedMessage,
) {
    let mut window_manager = window_manager.lock();

    window_manager.set_focused_app_window(focused_msg.window);
}
