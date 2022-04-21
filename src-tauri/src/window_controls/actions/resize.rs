use tauri::{Error, Manager};

use crate::window_controls::config::AppWindow;

pub fn resize_window(
    handle: &tauri::AppHandle,
    window_label: AppWindow,
    size: &tauri::LogicalSize<f64>,
) -> Result<(), Error> {
    if window_label == AppWindow::None {
        return Err(Error::WebviewNotFound);
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        app_window.set_size(tauri::Size::Logical(*size))
    } else {
        return Err(Error::WebviewNotFound);
    }
}
