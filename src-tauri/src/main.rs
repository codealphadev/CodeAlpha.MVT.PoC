#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::sync::Arc;
use tracing::error;

use core_engine::{CoreEngine, TextRange};
use parking_lot::Mutex;
use platform::macos::{setup_observers, xcode::actions::replace_range_with_clipboard_text, GetVia};
use tauri::{
    utils::assets::EmbeddedAssets, Context, Menu, MenuEntry, MenuItem, Submenu, SystemTrayEvent,
    SystemTrayMenuItem,
};
use tracing::{debug, info};
use window_controls::WindowManager;

use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};

mod core_engine;
mod platform;
mod utils;
mod window_controls;

use lazy_static::lazy_static;

use crate::{
    utils::{tracing::TracingSubscriber, updater::listen_for_updates},
    window_controls::{cmd_resize_window, cmd_toggle_app_activation},
};

use utils::feedback::cmd_send_feedback;
lazy_static! {
    static ref APP_HANDLE: Mutex<Option<tauri::AppHandle>> = Mutex::new(None);
}

lazy_static! {
    static ref NODE_EXPLAINATION_CURRENT_DOCSTRING: Arc<Mutex<String>> =
        Arc::new(Mutex::new("".to_string()));
}

lazy_static! {
    static ref NODE_EXPLAINATION_CURRENT_INSERTION_POINT: Arc<Mutex<usize>> =
        Arc::new(Mutex::new(0));
}

pub static CORE_ENGINE_ACTIVE_AT_STARTUP: bool = true;

fn set_static_app_handle(app_handle: &tauri::AppHandle) {
    APP_HANDLE.lock().replace(app_handle.clone());
}

pub fn app_handle() -> tauri::AppHandle {
    let app_handle = APP_HANDLE.lock().clone();

    app_handle.as_ref().unwrap().clone()
}

fn construct_tray_menu(context: &Context<EmbeddedAssets>) -> SystemTrayMenu {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let check_ax_api = CustomMenuItem::new("check_ax_api".to_string(), "Settings...");

    let version = context.package_info().version.clone();

    let version_label = CustomMenuItem::new(
        "version".to_string(),
        format!(
            "Version: {}.{}.{}",
            version.major, version.minor, version.patch
        )
        .as_str(),
    )
    .disabled();

    if !platform::macos::is_application_trusted() {
        SystemTrayMenu::new()
            .add_item(check_ax_api)
            .add_item(version_label)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit)
    } else {
        SystemTrayMenu::new()
            .add_item(version_label)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit)
    }
}

fn main() {
    // Configure tracing
    TracingSubscriber::new();

    let context = tauri::generate_context!("tauri.conf.json");

    let mut app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            cmd_toggle_app_activation,
            cmd_resize_window,
            cmd_send_feedback,
            cmd_paste_docs
        ])
        .setup(|app| {
            debug!(app_version = ?app.package_info().version);

            listen_for_updates(app.handle());

            // Set the app handle for the static APP_HANDLE variable
            set_static_app_handle(&app.handle());

            // Setup the observers for AX interactions and mouse events
            setup_observers();

            let core_engine_arc = Arc::new(Mutex::new(CoreEngine::new()));
            CoreEngine::start_core_engine_listeners(&core_engine_arc);

            // Start the window manager instance
            let window_manager = Arc::new(parking_lot::Mutex::new(WindowManager::new()?));
            WindowManager::start_event_listeners(&window_manager);

            // Continuously check if the accessibility APIs are enabled, show popup if not
            let ax_apis_enabled_at_start = platform::macos::is_application_trusted();
            tauri::async_runtime::spawn(async move {
                let mut popup_was_shown = false;
                loop {
                    let api_enabled;
                    if popup_was_shown {
                        api_enabled = platform::macos::is_application_trusted();
                    } else {
                        api_enabled = platform::macos::is_application_trusted_with_prompt();
                        popup_was_shown = true;
                    }

                    if api_enabled {
                        // In case AX apis were not enabled at program start, restart the app to
                        // ensure the AX observers are properly registered.
                        if !ax_apis_enabled_at_start {
                            app_handle().restart();
                        }
                    }

                    if !api_enabled && ax_apis_enabled_at_start {
                        // in this case the permissions were withdrawn at runtime, restart the app
                        std::process::exit(0);
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            });

            // Spin up a thread to detect potential Mutex deadlocks.
            deadlock_detection();

            Ok(())
        })
        .system_tray(SystemTray::new().with_menu(construct_tray_menu(&context)))
        .on_system_tray_event(|_app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    debug!("System tray: quit");
                    std::process::exit(0);
                }
                "check_ax_api" => {
                    debug!("System tray: check_ax_api");
                    platform::macos::is_application_trusted_with_prompt();
                }
                _ => {}
            },
            _ => {}
        })
        .menu(Menu::with_items([
            #[cfg(target_os = "macos")]
            MenuEntry::Submenu(Submenu::new(
                "dummy-menu-for-shortcuts-to-work-on-input-fields-see-github-issue-#-1055",
                Menu::with_items([
                    MenuItem::Undo.into(),
                    MenuItem::Redo.into(),
                    MenuItem::Cut.into(),
                    MenuItem::Copy.into(),
                    MenuItem::Paste.into(),
                    MenuItem::SelectAll.into(),
                ]),
            )),
        ]))
        .build(context)
        .expect("error while running tauri application");

    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        tauri::RunEvent::Updater(updater_event) => match updater_event {
            tauri::UpdaterEvent::DownloadProgress {
                chunk_length,
                content_length,
            } => {
                println!("downloaded {} of {:?}", chunk_length, content_length);
            }
            tauri::UpdaterEvent::UpdateAvailable {
                body,
                date,
                version,
            } => {
                info!("update available {} {:?} {}", body, date, version);
            }
            tauri::UpdaterEvent::Pending => {
                info!("update is pending!");
            }
            tauri::UpdaterEvent::Downloaded => {
                info!("update has been downloaded!");
            }
            tauri::UpdaterEvent::Updated => {
                info!("App has been updated");
            }
            tauri::UpdaterEvent::AlreadyUpToDate => {
                println!("app is already up to date");
            }
            tauri::UpdaterEvent::Error(error) => {
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

        println!("{} deadlocks detected", deadlocks.len());
        for (i, threads) in deadlocks.iter().enumerate() {
            println!("Deadlock #{}", i);
            for t in threads {
                println!("Thread Id {:#?}", t.thread_id());
                println!("{:#?}", t.backtrace());
            }
        }
    });
}

#[tauri::command]
fn cmd_paste_docs() {
    tauri::async_runtime::spawn(async move {
        // Paste it at the docs insertion point
        let insertion_point = NODE_EXPLAINATION_CURRENT_INSERTION_POINT.lock().clone();
        let docstring = NODE_EXPLAINATION_CURRENT_DOCSTRING.lock().clone();
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
        debug!(insertion_point, docstring, "Docstring inserted");
    });
}
