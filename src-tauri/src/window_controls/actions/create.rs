use tauri::{window::WindowBuilder, Error, Manager, Window, WindowUrl};
use window_shadows::set_shadow;

use crate::window_controls::config::{default_properties, AppWindow};

pub fn create_window(handle: &tauri::AppHandle, window_label: AppWindow) -> Result<Window, Error> {
    if window_label == AppWindow::None {
        return Err(Error::WebviewNotFound);
    }

    // If the window is already created, don't open it again.
    if let Some(window) = handle.get_window(&window_label.to_string()) {
        return Ok(window);
    }

    let window_builder = create_default_window_builder(&handle, window_label)?;

    let window = window_builder.build()?;

    if window_label == AppWindow::CodeOverlay {
        set_shadow(&window, false).expect("Unsupported platform!");
    } else {
        set_shadow(&window, true).expect("Unsupported platform!");
    }

    Ok(window)
}

pub fn create_default_window_builder(
    handle: &tauri::AppHandle,
    window_label: AppWindow,
) -> Result<WindowBuilder, Error> {
    if window_label == AppWindow::None {
        return Err(Error::WebviewNotFound);
    }

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
