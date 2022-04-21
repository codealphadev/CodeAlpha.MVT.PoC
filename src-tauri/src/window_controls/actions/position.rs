use tauri::{Error, LogicalPosition, Manager};

use crate::window_controls::config::AppWindow;

pub fn set_position(
    handle: &tauri::AppHandle,
    window_label: AppWindow,
    updated_position: &tauri::LogicalPosition<f64>,
) -> Result<(), Error> {
    if window_label == AppWindow::None {
        return Err(Error::WebviewNotFound);
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        // Only update if position changed
        if let (Ok(physical_outer), Ok(scale_factor)) =
            (app_window.outer_position(), app_window.scale_factor())
        {
            let logical_outer = physical_outer.to_logical::<f64>(scale_factor);
            if logical_outer != *updated_position {
                app_window.set_position(tauri::Position::Logical(*updated_position))?
            }
        } else {
            app_window.set_position(tauri::Position::Logical(*updated_position))?
        }
    }

    Ok(())
}

pub fn get_position(
    app_handle: &tauri::AppHandle,
    window_label: AppWindow,
) -> Result<LogicalPosition<f64>, Error> {
    if window_label == AppWindow::None {
        return Err(Error::WebviewNotFound);
    }

    if let Some(app_window) = app_handle.get_window(&window_label.to_string()) {
        if let Some(monitor) = app_window.current_monitor()? {
            let scale_factor = monitor.scale_factor();
            Ok(app_window.outer_position()?.to_logical::<f64>(scale_factor))
        } else {
            Err(Error::AssetNotFound("Monitor".to_string()))
        }
    } else {
        Err(Error::WebviewNotFound)
    }
}
