#![allow(non_snake_case)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ax_interaction::{
    setup_observers,
    xcode::{get_xcode_editor_content, update_xcode_editor_content},
};
use commands::search_and_replace_commands;
use tauri::{Manager, Menu, MenuEntry, MenuItem, Submenu, SystemTrayEvent};
use window_controls::{
    actions::{create_window, resize_window},
    AppWindow, EditorWindow, WidgetWindow, WindowStateManager,
};

use crate::window_controls::content_window::{
    cmd_resize_content_window, cmd_toggle_content_window,
};
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};

mod ax_interaction;
mod commands;
mod window_controls;

#[tauri::command]
fn cmd_resize_repair_window(app_handle: tauri::AppHandle, size_x: u32, size_y: u32) {
    let updated_content_size = tauri::LogicalSize {
        width: size_x as f64,
        height: size_y as f64,
    };

    let _ = resize_window(&app_handle, AppWindow::Repair, &updated_content_size);
}

#[tauri::command]
fn cmd_add_docstring(
    replace_str: String,
    widget_state: tauri::State<'_, Arc<Mutex<WidgetWindow>>>,
) {
    let widget_window = &*(widget_state.lock().unwrap());
    let editor_windows = &*(widget_window.editor_windows.lock().unwrap());

    if let Some(focused_editor_window_id) = widget_window.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows.get(&focused_editor_window_id) {
            let content = get_xcode_editor_content(editor_window.pid.try_into().unwrap());

            if let Ok(content) = content {
                if let Some(content_str) = content {
                    let content_str = format!("{}\n{}", replace_str, content_str);
                    let _ = update_xcode_editor_content(
                        editor_window.pid.try_into().unwrap(),
                        &content_str,
                    );
                }
            }
        }
    }
}

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
            cmd_resize_repair_window,
            cmd_add_docstring,
            cmd_toggle_content_window
        ])
        .setup(|app| {
            setup_observers(&app.handle());

            let handle = app.handle();

            // Create vector of editor windows
            let editor_windows_arc: Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>> =
                Arc::new(Mutex::new(HashMap::new()));

            let _ = create_window(&handle, AppWindow::Content);
            let _ = create_window(&handle, AppWindow::Repair);

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
            let handle_move_copy = app.handle().clone();
            let ax_apis_enabled_at_start = ax_interaction::application_is_trusted();
            tauri::async_runtime::spawn(async move {
                loop {
                    if ax_interaction::application_is_trusted_with_prompt() {
                        // In case AX apis were not enabled at program start, restart the app to
                        // ensure the AX observers are properly registered.
                        if !ax_apis_enabled_at_start {
                            handle_move_copy.restart();
                        }
                    }
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
