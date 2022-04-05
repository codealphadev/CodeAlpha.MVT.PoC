use tauri::{Manager, WindowUrl};

use crate::window_controls::{
    config::{default_properties, AppWindow},
    get_window_label, utils,
};

pub fn create_window(handle: tauri::AppHandle, window_type: AppWindow) {
    if window_type == AppWindow::None {
        return;
    }

    // If the window is already created, don't open it again.
    if handle.get_window(&get_window_label(window_type)).is_some() {
        return;
    }

    let parent_window = default_properties::parent_window(&window_type);
    let parent_window = handle.get_window(&utils::get_window_label(parent_window));

    if let Some(parent_window) = parent_window {
        if let Ok(parent) = parent_window.ns_window() {
            tauri::window::WindowBuilder::new(
                &handle,
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
            .parent_window(parent)
            .build()
            .unwrap();

            return;
        }
    }

    tauri::window::WindowBuilder::new(
        &handle,
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
    .center()
    .build()
    .unwrap();
}
