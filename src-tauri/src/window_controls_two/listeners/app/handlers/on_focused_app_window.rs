use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::app::AppWindowFocusedMessage,
    window_controls_two::{config::AppWindow, WindowManager},
};

pub fn on_focused_app_window(
    window_manager: &Arc<Mutex<WindowManager>>,
    focused_msg: &AppWindowFocusedMessage,
) {
    let mut window_manager = window_manager.lock();

    window_manager.set_focused_app_window(AppWindow::Widget);
    panic!("Change the AppWindow used in appactivate messge")
}
