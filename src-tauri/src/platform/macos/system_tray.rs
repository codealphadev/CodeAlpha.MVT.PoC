use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tracing::debug;

use crate::{
    app_handle,
    core_engine::events::{models::CoreActivationStatusMessage, EventUserInteraction},
    platform, TAURI_PACKAGE_INFO,
};

pub fn construct_system_tray() -> SystemTray {
    let tray_menu = construct_system_tray_menu(true);

    SystemTray::new().with_menu(tray_menu)
}

fn construct_system_tray_menu(core_engine_active: bool) -> SystemTrayMenu {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let check_ax_api = CustomMenuItem::new("check_ax_api".to_string(), "Settings...");

    let version;
    {
        let package_info_mutex = TAURI_PACKAGE_INFO.lock();
        version = package_info_mutex.as_ref().unwrap().version.clone();
    }

    let version_label = CustomMenuItem::new(
        "version".to_string(),
        format!(
            "Version: {}.{}.{}",
            version.major, version.minor, version.patch
        )
        .as_str(),
    )
    .disabled();

    let pause_core_engine =
        CustomMenuItem::new("pause_core_engine".to_string(), format!("⏸️ Pause").as_str());

    let resume_core_engine = CustomMenuItem::new(
        "resume_core_engine".to_string(),
        format!("▶️ Resume",).as_str(),
    );

    if !platform::macos::is_application_trusted() {
        SystemTrayMenu::new()
            .add_item(check_ax_api)
            .add_item(version_label)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit)
    } else {
        if core_engine_active {
            SystemTrayMenu::new()
                .add_item(pause_core_engine)
                .add_item(version_label)
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(quit)
        } else {
            SystemTrayMenu::new()
                .add_item(resume_core_engine)
                .add_item(version_label)
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(quit)
        }
    }
}

pub fn evaluate_system_tray_event(event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => on_quit(),
            "check_ax_api" => on_check_permissions(),
            "pause_core_engine" => on_pause_core_engine(),
            "resume_core_engine" => on_resume_core_engine(),
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

fn on_pause_core_engine() {
    debug!("System tray: PAUSE Pretzl");

    if app_handle()
        .tray_handle()
        .set_menu(construct_system_tray_menu(false))
        .is_ok()
    {
        EventUserInteraction::CoreActivationStatus(CoreActivationStatusMessage {
            engine_active: false,
        })
        .publish_to_tauri();
    }
}

fn on_resume_core_engine() {
    debug!("System tray: RESUME Pretzl");

    if app_handle()
        .tray_handle()
        .set_menu(construct_system_tray_menu(true))
        .is_ok()
    {
        EventUserInteraction::CoreActivationStatus(CoreActivationStatusMessage {
            engine_active: true,
        })
        .publish_to_tauri();
    }
}
