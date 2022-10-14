use tauri::{
    utils::assets::EmbeddedAssets, Context, CustomMenuItem, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem,
};
use tracing::debug;

use crate::platform;

pub fn construct_system_tray_menu(context: &Context<EmbeddedAssets>) -> SystemTray {
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

    let pause_core_engine = CustomMenuItem::new(
        "pause_core_engine".to_string(),
        format!("Pause Pretzl").as_str(),
    );

    let resume_core_engine = CustomMenuItem::new(
        "resume_core_engine".to_string(),
        format!("Resume Pretzl",).as_str(),
    );

    let tray_menu;
    if !platform::macos::is_application_trusted() {
        tray_menu = SystemTrayMenu::new()
            .add_item(check_ax_api)
            .add_item(version_label)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit)
    } else {
        tray_menu = SystemTrayMenu::new()
            .add_item(version_label)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit)
    }

    SystemTray::new().with_menu(tray_menu)
}

pub fn evaluate_system_tray_event(event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => on_quit(),
            "check_ax_api" => on_check_permissions(),
            _ => {}
        },
        _ => {}
    }
}

fn on_quit() {
    debug!("System tray: quit");
    std::process::exit(0);
}

fn on_check_permissions() {
    debug!("System tray: check_ax_api");
    platform::macos::is_application_trusted_with_prompt();
}
