use tauri::Manager;

use crate::window_controls::config::AppWindow;

pub fn set_position(
    handle: &tauri::AppHandle,
    window_label: AppWindow,
    updated_position: &tauri::LogicalPosition<f64>,
) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        // Only update if position changed
        if let (Ok(physical_outer), Ok(scale_factor)) =
            (app_window.outer_position(), app_window.scale_factor())
        {
            let logical_outer = physical_outer.to_logical::<f64>(scale_factor);
            if logical_outer != *updated_position {
                let _ = app_window.set_position(tauri::Position::Logical(*updated_position));
            }
        } else {
            let _ = app_window.set_position(tauri::Position::Logical(*updated_position));
        }
    }
}
