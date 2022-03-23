#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use plugins::xcode_state_plugin;
use tauri::Manager;
use tokio::time::{sleep, Duration};
use utils::{window_controls, xcode_twin::XCodeTwin};

mod plugins;
mod utils;
mod websocket;

static DEFAULT_AX_URL: &str = "ws://127.0.0.1:8080/channel";

#[tokio::main]
async fn main() {
    let url = url::Url::parse(&DEFAULT_AX_URL).expect("No valid URL path provided.");

    let app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            window_controls::cmd_open_window,
            window_controls::cmd_toggle_window,
            window_controls::cmd_close_window,
            window_controls::cmd_resize_window,
            window_controls::cmd_is_window_visible,
            utils::window_positioning::cmd_update_widget_position,
            utils::window_positioning::cmd_start_dragging_widget,
            utils::window_positioning::cmd_update_content_position
        ])
        .plugin(xcode_state_plugin::init())
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.manage(XCodeTwin::new(url, app.handle().clone()));

    // Load default windows
    window_controls::startup_windows(app.handle().clone());

    // Test: monitor window position all the time
    let handle_for_thread = app.handle().clone();
    tokio::spawn(async move {
        loop {
            utils::window_positioning::cmd_update_content_position(handle_for_thread.clone());
            // Sleep for 25ms to not drive up CPU load
            sleep(Duration::from_millis(25)).await;
        }
    });

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
