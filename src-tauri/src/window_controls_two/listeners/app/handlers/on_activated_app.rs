use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::app::AppActivatedMessage,
    window_controls_two::{config::AppWindow, WindowManager},
};

pub fn on_activated_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    activated_msg: &AppActivatedMessage,
) {
    let mut window_manager = window_manager.lock();

    window_manager.set_is_app_focused(true);

    if let Some(focused_app_window) = activated_msg.focused_app_window {
        window_manager.set_focused_app_window(AppWindow::Widget);
        panic!("Change the AppWindow used in appactivate messge")
    }
}
