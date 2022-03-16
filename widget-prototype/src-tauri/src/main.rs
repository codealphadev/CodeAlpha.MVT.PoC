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

#[tauri::command]
fn open_settings_window<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    println!("I was invoked from JS!");

    let _ = tauri::Builder::default().create_window(
        "Rust",
        tauri::WindowUrl::App("index.html".into()),
        |win, webview| {
            let win = win
                .title("Tauri - Rust")
                .resizable(true)
                .inner_size(800.0, 550.0)
                .min_inner_size(400.0, 200.0);
            return (win, webview);
        },
    );

    let _ = handle.create_window(
        "Rust".to_string(),
        tauri::WindowUrl::App("index.html".into()),
        |window_builder, webview_attributes| {
            (window_builder.title("Tauri - Rust"), webview_attributes)
        },
    );
}

#[tokio::main]
async fn main() {
    let url = url::Url::parse(&DEFAULT_AX_URL).expect("No valid URL path provided.");

    let app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_settings_window])
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
