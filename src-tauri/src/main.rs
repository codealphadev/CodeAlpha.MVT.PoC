#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use app_state::AppHandleExtension;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{ActivationPolicy, AppHandle, Builder, Menu, RunEvent, SystemTray, UpdaterEvent};
use tracing::{debug, error, info};

mod app_state;
mod core_engine;
mod platform;
mod utils;
mod window_controls;

use core_engine::CoreEngine;
use platform::macos::{
    menu::mac_os_task_bar_menu, permissions_check::ax_permissions_check, setup_observers,
    system_tray::evaluate_system_tray_event,
};
use utils::{feedback::cmd_send_feedback, tracing::TracingSubscriber};
use window_controls::{cmd_rebind_main_widget, cmd_resize_window, WindowManager};

#[cfg(not(debug_assertions))]
use crate::utils::updater::listen_for_updates;
use crate::{
    app_state::cmd_get_core_engine_state, core_engine::cmd_paste_docs,
    platform::macos::system_tray::construct_system_tray_menu,
};

lazy_static! {
    static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
}

pub static CORE_ENGINE_ACTIVE_AT_STARTUP: bool = true;

fn set_static_app_handle(app_handle: &AppHandle) {
    APP_HANDLE.lock().replace(app_handle.clone());
}

pub fn app_handle() -> AppHandle {
    let app_handle = APP_HANDLE.lock().clone();

    app_handle.as_ref().unwrap().clone()
}

fn main() {
    // Configure tracing
    TracingSubscriber::new();

    let tauri_context = tauri::generate_context!("tauri.conf.json");

    let mut app: tauri::App = Builder::default()
        .invoke_handler(tauri::generate_handler![
            cmd_resize_window,
            cmd_send_feedback,
            cmd_create_tests,
            cmd_rebind_main_widget,
            cmd_get_core_engine_state,
        ])
        .setup(|app| {
            debug!(app_version = ?app.package_info().version);

            #[cfg(not(debug_assertions))]
            listen_for_updates(app.handle());

            // Set the app handle for the static APP_HANDLE variable
            set_static_app_handle(&app.handle());

            // Load the app state
            _ = app.handle().load_core_engine_state();

            // Build system tray using the app handle; can't be done as part of the `.system_tray()` builder step
            // because we need the app handle to load the app state
            _ = app_handle()
                .tray_handle()
                .set_menu(construct_system_tray_menu());

            // Setup the observers for AX interactions and mouse events
            setup_observers();

            let core_engine_arc = Arc::new(Mutex::new(CoreEngine::new()));
            CoreEngine::start_core_engine_listeners(&core_engine_arc);

            // Start the window manager instance
            let window_manager = Arc::new(parking_lot::Mutex::new(WindowManager::new()?));
            WindowManager::start_event_listeners(&window_manager);

            ax_permissions_check();

            // Spin up a thread to detect potential Mutex deadlocks.
            deadlock_detection();

            Ok(())
        })
        .system_tray(SystemTray::new())
        .on_system_tray_event(|_, event| evaluate_system_tray_event(event))
        .menu(Menu::with_items([mac_os_task_bar_menu()]))
        .build(tauri_context)
        .expect("error while running tauri application");

    app.set_activation_policy(ActivationPolicy::Accessory);
    app.run(|_, event| match event {
        RunEvent::Updater(updater_event) => match updater_event {
            UpdaterEvent::DownloadProgress {
                chunk_length,
                content_length,
            } => {
                println!("downloaded {} of {:?}", chunk_length, content_length);
            }
            UpdaterEvent::UpdateAvailable {
                body,
                date,
                version,
            } => {
                info!("update available {} {:?} {}", body, date, version);
            }
            UpdaterEvent::Pending => {
                info!("update is pending!");
            }
            UpdaterEvent::Downloaded => {
                info!("update has been downloaded!");
            }
            UpdaterEvent::Updated => {
                info!("App has been updated");
            }
            UpdaterEvent::AlreadyUpToDate => {}
            UpdaterEvent::Error(error) => {
                error!(?error, "Failed to update");
            }
        },
        _ => {}
    });
}

fn deadlock_detection() {
    use parking_lot::deadlock;
    use std::thread;
    use std::time::Duration;

    // Create a background thread which checks for deadlocks every 2s
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));
        let deadlocks = deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        for (_, threads) in deadlocks.iter().enumerate() {
            for t in threads {
                error!(
                    thread_id = t.thread_id(),
                    backtrace = format!("{:?}", t.backtrace()),
                    "Deadlock Detected"
                );
            }
        }
    });
}

#[tauri::command]
pub fn cmd_create_tests() {
    tauri::async_runtime::spawn(async move {
        // Paste it at the docs insertion point
        let insertion_point = NODE_EXPLANATION_CURRENT_INSERTION_POINT.lock().clone();
        let docstring = NODE_EXPLANATION_CURRENT_DOCSTRING.lock().clone();
        replace_range_with_clipboard_text(
            &app_handle(),
            &GetVia::Current,
            &TextRange {
                index: insertion_point,
                length: 0,
            },
            Some(&docstring),
            true,
        )
        .await;
    });
}
