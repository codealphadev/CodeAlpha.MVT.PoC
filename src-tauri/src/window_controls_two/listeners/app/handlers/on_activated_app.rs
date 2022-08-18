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
        window_manager
            .set_focused_app_window(convert_app_window_to_app_window_type(focused_app_window));
    }
}

fn convert_app_window_to_app_window_type(
    app_window: crate::window_controls::config::AppWindow,
) -> AppWindow {
    match app_window {
        crate::window_controls::config::AppWindow::Settings => AppWindow::Settings,
        crate::window_controls::config::AppWindow::Analytics => AppWindow::Analytics,
        crate::window_controls::config::AppWindow::Widget => AppWindow::Widget,
        crate::window_controls::config::AppWindow::Content => AppWindow::Content,
        crate::window_controls::config::AppWindow::Repair => AppWindow::Repair,
        crate::window_controls::config::AppWindow::CodeOverlay => AppWindow::CodeOverlay,
        crate::window_controls::config::AppWindow::None => AppWindow::Widget,
    }
}
