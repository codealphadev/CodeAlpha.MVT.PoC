#![allow(unused)]

use tauri::{Error, Manager};

use crate::{app_handle, window_controls::config::AppWindow};

use super::geometry::{LogicalFrame, LogicalPosition, LogicalSize};

pub fn get_tauri_window_frame(app_window: &AppWindow) -> Result<LogicalFrame, Error> {
    let tauri_window =
        app_handle()
            .get_window(&app_window.to_string())
            .ok_or(tauri::Error::AssetNotFound(
                format!("Tauri API call failed to obtain window: {}", app_window).into(),
            ))?;
    let monitor = tauri_window
        .current_monitor()?
        .ok_or(tauri::Error::AssetNotFound(format!(
            "Tauri API call failed to obtain screen for window: {}",
            app_window
        )))?;
    let scale_factor = monitor.scale_factor();
    let origin = tauri_window
        .outer_position()?
        .to_logical::<f64>(scale_factor);
    let size = tauri_window.outer_size()?.to_logical::<f64>(scale_factor);

    Ok(LogicalFrame {
        origin: LogicalPosition::from_tauri_LogicalPosition(&origin),
        size: LogicalSize::from_tauri_LogicalSize(&size),
    })
}
