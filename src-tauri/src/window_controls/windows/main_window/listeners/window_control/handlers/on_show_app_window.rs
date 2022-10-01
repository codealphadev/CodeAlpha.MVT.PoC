use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;
use tracing::debug;

use crate::{
    app_handle,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::AppWindow, events::models::app_window::ShowAppWindowMessage, windows::MainWindow,
    },
};

pub fn on_show_app_window(
    main_window: &Arc<Mutex<MainWindow>>,
    show_msg: &ShowAppWindowMessage,
) -> Option<()> {
    if show_msg.app_windows.contains(&AppWindow::Main) {
        let main_window = main_window.lock();

        // We fetch the window where the widget is on
        let widget_window = app_handle().get_window(&AppWindow::Widget.to_string())?;
        let monitor = widget_window.current_monitor().ok()??;

        let scale_factor = monitor.scale_factor();
        let origin = LogicalPosition::from_tauri_LogicalPosition(
            &monitor.position().to_logical::<f64>(scale_factor),
        );
        let size =
            LogicalSize::from_tauri_LogicalSize(&monitor.size().to_logical::<f64>(scale_factor));

        if main_window.show(&LogicalFrame { origin, size }).is_none() {
            debug!("Failed to show MainWindow (on_show_app_window.rs)");
        }
    }

    Some(())
}
