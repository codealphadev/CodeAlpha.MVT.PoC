#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use plugins::websocket_plugin;
use utils::xcode_twin::XCodeTwin;

mod plugins;
mod utils;
mod websocket;

#[tokio::main]
async fn main() {
    let app: tauri::App = tauri::Builder::default()
        .plugin(websocket_plugin::init())
        .manage(XCodeTwin::new())
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
