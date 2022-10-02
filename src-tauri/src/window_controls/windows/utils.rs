use tauri::Manager;

use crate::{
    app_handle,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::config::AppWindow,
};

pub fn app_window_dimensions(window: AppWindow) -> LogicalFrame {
    let tauri_window = app_handle()
        .get_window(&window.to_string())
        .expect(&format!("Could not get window: {:?}!", window.to_string()));

    let scale_factor = tauri_window.scale_factor().expect(&format!(
        "Could not get window: {:?} scale factor!",
        window.to_string()
    ));
    let widget_position = LogicalPosition::from_tauri_LogicalPosition(
        &tauri_window
            .outer_position()
            .expect(&format!(
                "Could not get window: {:?} outer position!",
                window.to_string()
            ))
            .to_logical::<f64>(scale_factor),
    );
    let widget_size = LogicalSize::from_tauri_LogicalSize(
        &tauri_window
            .outer_size()
            .expect(&format!(
                "Could not get window: {:?} outer size!",
                window.to_string()
            ))
            .to_logical::<f64>(scale_factor),
    );

    LogicalFrame {
        origin: widget_position,
        size: widget_size,
    }
}
