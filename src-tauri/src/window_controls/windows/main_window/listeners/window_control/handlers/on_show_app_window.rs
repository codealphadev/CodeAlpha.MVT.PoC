use std::sync::Arc;

use parking_lot::Mutex;
use tracing::debug;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::ShowAppWindowMessage, windows::MainWindow,
};

pub fn on_show_app_window(
    main_window: &Arc<Mutex<MainWindow>>,
    show_msg: &ShowAppWindowMessage,
) -> Option<()> {
    if show_msg.app_windows.contains(&AppWindow::Main) {
        let main_window = main_window.lock();

        if main_window.show(&show_msg.monitor).is_none() {
            debug!("Failed to show MainWindow (on_show_app_window.rs)");
        }
    }

    Some(())
}
