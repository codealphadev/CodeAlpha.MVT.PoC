#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use plugins::websocket_plugin;

mod plugins;
mod websocket;

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    let app: tauri::App = tauri::Builder::default()
        .plugin(websocket_plugin::init())
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
