use tauri::{Error, Manager, Window, WindowUrl};

use crate::window_controls::{
    config::{default_properties, AppWindow},
    get_window_label, special_default_position_for_content_window, utils,
};

pub fn create_window(handle: &tauri::AppHandle, window_type: AppWindow) -> Result<Window, Error> {
    if window_type == AppWindow::None {
        return Err(Error::CreateWindow);
    }

    // If the window is already created, don't open it again.
    if let Some(window) = handle.get_window(&get_window_label(window_type)) {
        return Ok(window);
    }

    let mut window_builder = tauri::window::WindowBuilder::new(
        handle,
        get_window_label(window_type),
        WindowUrl::App(default_properties::url(&window_type).into()),
    )
    .title(default_properties::title(&window_type))
    .inner_size(
        default_properties::size(&window_type).0,
        default_properties::size(&window_type).1,
    )
    .resizable(default_properties::is_resizable(&window_type))
    .transparent(default_properties::is_transparent(&window_type))
    .decorations(default_properties::has_decorations(&window_type))
    .visible(default_properties::is_visible(&window_type))
    .always_on_top(default_properties::is_always_on_top(&window_type))
    .skip_taskbar(default_properties::skip_taskbar(&window_type));

    match window_type {
        AppWindow::Content => {
            // Add Parent Window
            let parent_window = default_properties::parent_window(&window_type);
            let parent_window = handle.get_window(&utils::get_window_label(parent_window));
            if let Some(parent_window) = parent_window {
                if let Ok(parent) = parent_window.ns_window() {
                    window_builder = window_builder.parent_window(parent);
                }
            }

            // Set position
            if let Ok(result) = special_default_position_for_content_window(&handle) {
                if let Some(position) = result {
                    window_builder = window_builder.position(position.0, position.1);
                }
            }
        }
        _ => {}
    }

    if default_properties::initially_focused(&window_type) {
        window_builder = window_builder.focus();
    }

    window_builder.build()
}
