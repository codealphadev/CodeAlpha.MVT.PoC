use serde::{Deserialize, Serialize};
use tauri::{window::WindowBuilder, Manager, WindowBuilder as DeprecatedWindowBuilder, WindowUrl};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
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

pub fn startup_windows<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    let startup_window_list: [AppWindow; 2] = [AppWindow::Widget, AppWindow::Content];

    for window_label in startup_window_list.iter() {
        if *window_label == AppWindow::None {
            continue;
        }

        // If the window is already created, don't open it again.
        if handle.get_window(&window_label.to_string()).is_some() {
            continue;
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
}

#[allow(deprecated)]
#[tauri::command]
pub fn open_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        let _ = app_window.show();
    } else {
        let url_slug = format!("{}{}", r"\", window_label.to_string().to_lowercase());
        let window_title = format!("CodeAlpha - {}", window_label.to_string());
        let (size_x, size_y) = get_window_size(&window_label);
        let (resizable, transparent, decorations, _) = get_window_features(&window_label);

        let _ = handle.create_window(
            window_label.to_string(),
            tauri::WindowUrl::App(url_slug.into()),
            |window_builder, webview_attributes| {
                (
                    window_builder
                        .title(window_title)
                        .inner_size(size_x, size_y)
                        .resizable(resizable)
                        .transparent(transparent)
                        .decorations(decorations),
                    webview_attributes,
                )
            },
        );
    }
}

#[tauri::command]
pub fn close_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        let _ = app_window.hide();
    }
}

#[tauri::command]
pub fn toggle_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>, window_label: AppWindow) {
    if window_label == AppWindow::None {
        return;
    }

    let app_window = handle.get_window(&window_label.to_string());

    if let Some(app_window) = app_window {
        if app_window.is_visible().unwrap() {
            close_window(handle.clone(), window_label);
        } else {
            open_window(handle.clone(), window_label);
        }
    } else {
        open_window(handle.clone(), window_label);
    }
}

#[tauri::command]
pub fn resize_window<R: tauri::Runtime>(
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

fn get_window_size(window: &AppWindow) -> (f64, f64) {
    match window {
        AppWindow::Settings => (800.0, 600.0),
        AppWindow::Analytics => (1280.0, 786.0),
        AppWindow::Widget => (48.0, 48.0),
        AppWindow::Content => (400.0, 316.0),
        AppWindow::None => (0.0, 0.0),
    }
}

fn get_window_features(window: &AppWindow) -> (bool, bool, bool, bool) {
    // resizable, transparent, decorations, visible
    match window {
        AppWindow::Settings => (false, false, true, true),
        AppWindow::Analytics => (true, false, true, true),
        AppWindow::Widget => (false, true, false, true),
        AppWindow::Content => (false, true, false, false),
        AppWindow::None => (false, false, false, true),
    }
}
