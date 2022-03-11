#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use plugins::xcode_state_plugin;
use utils::xcode_twin::XCodeTwin;

mod plugins;
mod utils;
mod websocket;

static DEFAULT_AX_URL: &str = "ws://127.0.0.1:8080/channel";

#[tokio::main]
async fn main() {
    let url = url::Url::parse(&DEFAULT_AX_URL).expect("No valid URL path provided.");

    let app: tauri::App = tauri::Builder::default()
        .plugin(xcode_state_plugin::init())
        .manage(XCodeTwin::new(url))
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
