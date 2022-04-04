use serde::{Deserialize, Serialize};
use tauri::{window::WindowBuilder, Manager, WindowUrl};

use super::window_positioning::{self};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AppWindow {
    Settings,
    Analytics,
    Widget,
    Content,
    None,
}

impl std::fmt::Display for AppWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Definition of initial sizes for all app windows
fn get_window_size(window: &AppWindow) -> (f64, f64) {
    match window {
        AppWindow::Settings => (800.0, 600.0),
        AppWindow::Analytics => (1280.0, 786.0),
        AppWindow::Widget => (48.0, 48.0),
        AppWindow::Content => (322.0, 316.0),
        AppWindow::None => (0.0, 0.0),
    }
}

// Definition of initial set of properties for all app windows
fn get_window_features(window: &AppWindow) -> (bool, bool, bool, bool) {
    // resizable, transparent, decorations, visible
    match window {
        AppWindow::Settings => (false, false, true, true),
        AppWindow::Analytics => (true, false, true, true),
        AppWindow::Widget => (false, true, false, false),
        AppWindow::Content => (false, true, false, false),
        AppWindow::None => (false, false, false, true),
    }
}

#[tauri::command]
pub fn cmd_is_window_visible<R: tauri::Runtime>(
    handle: tauri::AppHandle<R>,
    window_label: AppWindow,
) -> bool {
    if window_label == AppWindow::None {
        return false;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        return app_window.is_visible().unwrap();
    } else {
        return false;
    }
}

#[tauri::command]
pub fn cmd_open_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        let _ = app_window.show();
    } else {
        create_window(handle, window_label);
    }
}

#[tauri::command]
pub fn cmd_close_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        let _ = app_window.hide();
    }
}

#[tauri::command]
pub fn cmd_toggle_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        if app_window.is_visible().unwrap() {
            cmd_close_window(handle.clone(), window_label);
        } else {
            cmd_open_window(handle.clone(), window_label);
        }
    } else {
        cmd_open_window(handle.clone(), window_label);
    }
}

#[tauri::command]
pub fn cmd_resize_window<R: tauri::Runtime>(
    handle: tauri::AppHandle<R>,
    window_label: AppWindow,
    size_x: u32,
    size_y: u32,
) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        let new_size = tauri::LogicalSize {
            width: size_x as f64,
            height: size_y as f64,
        };

        let _ = app_window.set_size(tauri::Size::Logical(new_size));
    }
}

pub fn startup_windows<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    let startup_window_list: [AppWindow; 2] = [AppWindow::Widget, AppWindow::Content];

    for window_label in startup_window_list.iter() {
        create_window(handle.clone(), *window_label);
    }

    // position the content window in relation to the widget
    window_positioning::cmd_update_content_position(handle.clone());
}

fn create_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    // If the window is already created, don't open it again.
    if handle.get_window(&window_label.to_string()).is_some() {
        return;
    }

    // Get window properties
    let url_slug = format!("{}{}", r"\", window_label.to_string().to_lowercase());
    let title = format!("CodeAlpha - {}", window_label.to_string());
    let (size_x, size_y) = get_window_size(&window_label);
    let (resizable, transparent, decorations, visible) = get_window_features(&window_label);

    WindowBuilder::new(
        &handle,
        window_label.to_string(),
        WindowUrl::App(url_slug.into()),
    )
    .title(title)
    .inner_size(size_x, size_y)
    .resizable(resizable)
    .transparent(transparent)
    .decorations(decorations)
    .visible(visible)
    .center()
    .build()
    .unwrap();
}
