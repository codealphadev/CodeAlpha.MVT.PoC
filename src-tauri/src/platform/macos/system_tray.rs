use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{CustomMenuItem, Manager, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tracing::debug;

use crate::{
    app_handle,
    app_state::{CoreEngineState, CoreEngineStateCache},
    core_engine::events::{models::AiFeaturesStatusMessage, EventUserInteraction},
    platform, TAURI_PACKAGE_INFO,
};

use crate::app_state::AppHandleExtension;

pub fn construct_system_tray_menu() -> SystemTrayMenu {
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

    let pause_ai_features = CustomMenuItem::new(
        "pause_ai_features".to_string(),
        format!("⏸️ Pause AI Features").as_str(),
    );

    let resume_ai_features = CustomMenuItem::new(
        "resume_ai_features".to_string(),
        format!("▶️ Resume AI Features",).as_str(),
    );

    if !platform::macos::is_application_trusted() {
        SystemTrayMenu::new()
            .add_item(check_ax_api)
            .add_item(version_label)
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit)
    } else {
        if check_ai_features() {
            SystemTrayMenu::new()
                .add_item(pause_ai_features)
                .add_item(version_label)
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(quit)
        } else {
            SystemTrayMenu::new()
                .add_item(resume_ai_features)
                .add_item(version_label)
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(quit)
        }
    }
}

fn check_ai_features() -> bool {
    if let Some(cache) = app_handle().try_state::<CoreEngineStateCache>() {
        cache.0.lock().ai_features_active
    } else {
        true
    }
}

pub fn evaluate_system_tray_event(event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => on_quit(),
            "check_ax_api" => on_check_permissions(),
            "pause_ai_features" => on_pause_ai_features(),
            "resume_ai_features" => on_resume_ai_features(),
            _ => {}
        },
        _ => {}
    }
}

fn on_quit() {
    debug!("System tray: quit");
    _ = app_handle().save_core_engine_state();
    app_handle().exit(0);
}

fn on_check_permissions() {
    debug!("System tray: check_ax_api");
    platform::macos::is_application_trusted_with_prompt();
}

fn on_pause_ai_features() {
    debug!("System tray: PAUSE AI Features");

    update_app_state_ai_features_active(false);

    if app_handle()
        .tray_handle()
        .set_menu(construct_system_tray_menu())
        .is_ok()
    {
        EventUserInteraction::AiFeaturesStatus(AiFeaturesStatusMessage {
            ai_features_active: check_ai_features(),
        })
        .publish_to_tauri();
    }
}

fn on_resume_ai_features() {
    debug!("System tray: RESUME AI Features");

    update_app_state_ai_features_active(true);

    if app_handle()
        .tray_handle()
        .set_menu(construct_system_tray_menu())
        .is_ok()
    {
        EventUserInteraction::AiFeaturesStatus(AiFeaturesStatusMessage {
            ai_features_active: check_ai_features(),
        })
        .publish_to_tauri();
    }
}

fn update_app_state_ai_features_active(ai_features_active: bool) {
    if let Some(cache) = app_handle().try_state::<CoreEngineStateCache>() {
        let current_core_engine_state = cache.0.lock().clone();
        *cache.0.lock() = CoreEngineState {
            ai_features_active,
            ..current_core_engine_state
        };
    } else {
        let cache = Arc::new(Mutex::new(CoreEngineState {
            ai_features_active,
            ..Default::default()
        }));
        app_handle().manage(CoreEngineStateCache(cache));
    };
}
