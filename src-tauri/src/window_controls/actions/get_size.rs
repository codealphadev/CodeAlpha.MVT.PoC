use tauri::{Error, LogicalSize, Manager};

use crate::window_controls::config::AppWindow;

pub fn get_size(
    app_handle: &tauri::AppHandle,
    window_label: AppWindow,
) -> Result<LogicalSize<f64>, Error> {
    if window_label == AppWindow::None {
        return Err(Error::WebviewNotFound);
    }

    if let Some(app_window) = app_handle.get_window(&window_label.to_string()) {
        if let Some(monitor) = app_window.current_monitor()? {
            let scale_factor = monitor.scale_factor();
            Ok(app_window.outer_size()?.to_logical::<f64>(scale_factor))
        } else {
            Err(Error::AssetNotFound("Monitor".to_string()))
        }
    } else {
        Err(Error::WebviewNotFound)
    }
}
