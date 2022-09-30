use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{
    config::AppWindow, events::models::app_window::HideAppWindowMessage, windows::MainWindow,
};

pub fn on_hide_app_window(
    main_window: &Arc<Mutex<MainWindow>>,
    hide_msg: &HideAppWindowMessage,
) -> Option<()> {
    if hide_msg.app_windows.contains(&AppWindow::Main) {
        let main_window = main_window.lock();

        if main_window.hide().is_none() {
            println!("Failed to hide main window");
        };
    }

    Some(())
}
