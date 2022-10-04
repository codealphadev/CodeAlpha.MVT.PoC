use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;
use tracing::debug;

use crate::{
    app_handle,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::AppWindow, models::app_window::UpdateAppWindowMessage, windows::MainWindow,
    },
};

pub fn on_update_app_window(
    _main_window: &Arc<Mutex<MainWindow>>,
    update_msg: &UpdateAppWindowMessage,
) -> Option<()> {
    println!("on_update_app_window");
    if update_msg.app_windows.contains(&AppWindow::Main) {
        if let Some(main_window_size) = update_msg.window_size {
            println!("on_update_app_window - 2");
            // We fetch the window where the widget is on
            let main_tauri_window = app_handle().get_window(&AppWindow::Main.to_string())?;
            let monitor = main_tauri_window.current_monitor().ok()??;

            let scale_factor = monitor.scale_factor();
            let monitor_origin = LogicalPosition::from_tauri_LogicalPosition(
                &monitor.position().to_logical::<f64>(scale_factor),
            );
            let monitor_size = LogicalSize::from_tauri_LogicalSize(
                &monitor.size().to_logical::<f64>(scale_factor),
            );
            dbg!(main_window_size);

            if MainWindow::update(
                &main_window_size,
                &LogicalFrame {
                    origin: monitor_origin,
                    size: monitor_size,
                },
            )
            .is_none()
            {
                debug!("Failed to update MainWindow");
            }
        }
    }

    Some(())
}
