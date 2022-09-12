use cocoa::base::id;
use objc::{msg_send, sel, sel_impl};
use tauri::{window::WindowBuilder, Error, Manager, WindowUrl};

use crate::{
    app_handle,
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls::config::{default_properties, AppWindow},
};

pub fn get_window_level(window_label: AppWindow) -> Option<i64> {
    let tauri_window = app_handle().get_window(&window_label.to_string())?;

    if let Ok(ns_window_ptr) = tauri_window.ns_window() {
        unsafe {
            let window_level: i64 = msg_send![ns_window_ptr as id, level];
            return Some(window_level);
        }
    }

    None
}

pub fn get_position(window_label: AppWindow) -> Option<LogicalPosition> {
    let tauri_window = app_handle().get_window(&window_label.to_string())?;
    let monitor = tauri_window.current_monitor().ok()??;
    let scale_factor = monitor.scale_factor();

    Some(LogicalPosition::from_tauri_LogicalPosition(
        &tauri_window
            .outer_position()
            .ok()?
            .to_logical::<f64>(scale_factor),
    ))
}

pub fn get_size(window_label: AppWindow) -> Option<LogicalSize> {
    let tauri_window = app_handle().get_window(&window_label.to_string())?;
    let monitor = tauri_window.current_monitor().ok()??;
    let scale_factor = monitor.scale_factor();

    Some(LogicalSize::from_tauri_LogicalSize(
        &tauri_window
            .outer_size()
            .ok()?
            .to_logical::<f64>(scale_factor),
    ))
}

pub fn create_default_window_builder(
    handle: &tauri::AppHandle,
    window_label: AppWindow,
) -> Result<WindowBuilder, Error> {
    let window_builder = tauri::window::WindowBuilder::new(
        handle,
        window_label.to_string(),
        WindowUrl::App(default_properties::url(&window_label).into()),
    )
    .title(default_properties::title(&window_label))
    .inner_size(
        default_properties::size(&window_label).0,
        default_properties::size(&window_label).1,
    )
    .resizable(default_properties::is_resizable(&window_label))
    .transparent(default_properties::is_transparent(&window_label))
    .decorations(default_properties::has_decorations(&window_label))
    .visible(default_properties::is_visible(&window_label))
    .always_on_top(default_properties::is_always_on_top(&window_label))
    .skip_taskbar(default_properties::skip_taskbar(&window_label));

    Ok(window_builder)
}
