#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};

use ax_interaction::setup_observers;
use commands::search_and_replace_commands;
use tauri::{Manager, Menu, MenuEntry, MenuItem, Submenu, SystemTrayEvent};
use window_controls::{
    actions::create_window, AppWindow, EditorWindow, WidgetWindow, WindowStateManager,
};

use crate::window_controls::content_window::{
    cmd_resize_content_window, cmd_toggle_content_window,
};
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};

mod ax_interaction;
mod commands;
mod window_controls;

fn main() {
    // Configure system tray
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new().add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    let mut app: tauri::App = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search_and_replace_commands::cmd_search_and_replace,
            cmd_resize_content_window,
            cmd_toggle_content_window
        ])
        .setup(|app| {
            setup_observers(&app.handle());

            let handle = app.handle();

            // Create vector of editor windows
            let editor_windows_arc: Arc<Mutex<Vec<EditorWindow>>> =
                Arc::new(Mutex::new(Vec::new()));

            let _ = create_window(&handle, AppWindow::Content);

            // Create instance of widget window; panics if creation fails
            let widget_window = WidgetWindow::new(&handle, &editor_windows_arc);
            let widget_window_arc = Arc::new(Mutex::new(widget_window));
            WidgetWindow::setup_widget_listeners(&handle, &widget_window_arc);

            let _state_manager = WindowStateManager::new(
                &handle,
                editor_windows_arc.clone(),
                widget_window_arc.clone(),
            );

            app.manage(widget_window_arc);

            // Continuously check if the accessibility APIs are enabled, show popup if not
            tauri::async_runtime::spawn(async {
                loop {
                    if !ax_interaction::application_is_trusted_with_prompt() {}
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            });

            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|_app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
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
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("error while running tauri application");

    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });
}
