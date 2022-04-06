#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ax_interaction::{setup_observers, utils::TauriState};
use commands::search_and_replace_commands;
use tauri::Manager;
use utils::window_state_machine::WindowStateMachine;

use crate::commands::window_control_commands;

mod ax_events;
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
            setup_observers(TauriState {
                handle: app.handle().clone(),
            });

            let mut window_state_machine = WindowStateMachine::new(app.handle().clone());
            window_state_machine.setup();
            app.manage(window_state_machine);

            let window_state = window_controls::WindowStateManager::new(app.handle().clone());
            window_state.launch_startup_windows();

            // Continuously check if the accessibility APIs are enabled
            std::thread::spawn(|| loop {
                if !ax_interaction::application_is_trusted_with_prompt() {}
                std::thread::sleep(std::time::Duration::from_secs(5));
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
