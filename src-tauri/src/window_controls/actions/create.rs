use tauri::{Error, Manager, Window, WindowUrl};

use crate::window_controls::{
    config::{default_properties, AppWindow},
    get_window_label, special_default_position_for_content_window, utils,
};

pub fn create_window(handle: &tauri::AppHandle, window_label: AppWindow) -> Result<Window, Error> {
    if window_label == AppWindow::None {
        return Err(Error::CreateWindow);
    }

    // If the window is already created, don't open it again.
    if let Some(window) = handle.get_window(&get_window_label(window_label)) {
        return Ok(window);
    }

    let mut window_builder = tauri::window::WindowBuilder::new(
        handle,
        get_window_label(window_label),
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

    // Additional special logic for each window type
    match window_label {
        AppWindow::Content => {
            // Add Parent Window
            let parent_window = default_properties::parent_window(&window_label);
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

    window_builder.build()
}
