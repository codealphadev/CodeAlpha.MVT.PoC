#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ax_interaction::setup_observers;
use commands::search_and_replace_commands;

use crate::commands::window_control_commands;

mod ax_events_deprecated;
mod ax_interaction;
mod commands;
mod utils;
mod window_controls;

fn main() {
    let app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            window_control_commands::cmd_open_window,
            window_control_commands::cmd_toggle_window,
            window_control_commands::cmd_close_window,
            window_control_commands::cmd_resize_window,
            window_control_commands::cmd_is_window_visible,
            utils::window_positioning::cmd_update_widget_position,
            utils::window_positioning::cmd_start_dragging_widget,
            utils::window_positioning::cmd_update_content_position,
            search_and_replace_commands::cmd_search_and_replace
        ])
        .setup(|app| {
            setup_observers(&app.handle());

            let handle = app.handle();

            // Should panic if widget window fails do be created, hence the unwrap.
            let mut manager = window_controls::WindowStateManager::new(&handle);
            manager.configure_windows();

            // Continuously check if the accessibility APIs are enabled, show popup if not
            tauri::async_runtime::spawn(async {
                loop {
                    if !ax_interaction::application_is_trusted_with_prompt() {}
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
