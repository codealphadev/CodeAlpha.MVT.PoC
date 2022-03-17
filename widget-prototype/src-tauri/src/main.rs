#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use plugins::xcode_state_plugin;
use tauri::{Manager, WindowBuilder};
use utils::xcode_twin::XCodeTwin;

mod plugins;
mod utils;
mod websocket;

static DEFAULT_AX_URL: &str = "ws://127.0.0.1:8080/channel";

#[allow(deprecated)]
#[tauri::command]
fn open_settings_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    let _ = handle.create_window(
        "Rust".to_string(),
        tauri::WindowUrl::App("/settings".into()),
        |window_builder, webview_attributes| {
            (
                window_builder.title("CodeAlpha - Settings"),
                webview_attributes,
            )
        },
    );
}

#[allow(deprecated)]
#[tauri::command]
fn open_content_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    let content_window = handle.get_window("Content");

    if let Some(content_window) = content_window {
        let _ = content_window.show();
    } else {
        let _ = handle.create_window(
            "Content".to_string(),
            tauri::WindowUrl::App("/content".into()),
            |window_builder, webview_attributes| {
                (
                    window_builder
                        .title("CodeAlpha - Content")
                        .inner_size(384.0 + 2.0 * 16.0, 524.0)
                        .resizable(false)
                        .focus()
                        .transparent(true)
                        .decorations(false),
                    webview_attributes,
                )
            },
        );
    }
}

#[tauri::command]
fn close_content_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    let content_window = handle.get_window("Content");

    if let Some(content_window) = content_window {
        let _ = content_window.hide();
    }
}

#[tauri::command]
fn toggle_content_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    let content_window = handle.get_window("Content");

    if let Some(content_window) = content_window {
        if content_window.is_visible().unwrap() {
            close_content_window(handle.clone());
        } else {
            open_content_window(handle.clone());
        }
    } else {
        open_content_window(handle.clone());
    }
}

#[tauri::command]
fn resize_content_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>, size_x: u32, size_y: u32) {
    let content_window = handle.get_window("Content");

    if let Some(content_window) = content_window {
        let _ = content_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: size_x as f64,
            height: size_y as f64,
        }));
    }
}

#[tokio::main]
async fn main() {
    let url = url::Url::parse(&DEFAULT_AX_URL).expect("No valid URL path provided.");

    let app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_settings_window,
            toggle_content_window,
            open_content_window,
            close_content_window,
            resize_content_window
        ])
        .plugin(xcode_state_plugin::init())
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.manage(XCodeTwin::new(url, app.handle().clone()));

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
