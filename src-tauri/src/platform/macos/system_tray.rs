use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{CustomMenuItem, Manager, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tracing::debug;

use crate::{
    app_handle,
    app_state::{CoreEngineState, CoreEngineStateCache},
    core_engine::events::{models::AiFeaturesStatusMessage, EventUserInteraction},
    platform,
};

use crate::app_state::AppHandleExtension;

const TRAY_MENU_ACTIVATE_AI_FEATURES: &str = "activate_ai_features";
const TRAY_MENU_DEACTIVATE_AI_FEATURES: &str = "deactivate_ai_features";
const TRAY_MENU_EXIT: &str = "quit";
const TRAY_MENU_CHECK_AX_API: &str = "check_ax_api";
const TRAY_MENU_VERSION: &str = "version";

pub fn construct_system_tray_menu() -> SystemTrayMenu {
    let quit = CustomMenuItem::new(TRAY_MENU_EXIT.to_string(), "Quit");
    let check_ax_api = CustomMenuItem::new(TRAY_MENU_CHECK_AX_API.to_string(), "Settings...");

    let package_info = app_handle().package_info().clone();

    let version_label = CustomMenuItem::new(
        TRAY_MENU_VERSION.to_string(),
        format!(
            "Version: {}.{}.{}",
            package_info.version.major, package_info.version.minor, package_info.version.patch
        )
        .as_str(),
    )
    .disabled();

    let deactivate_ai_features = CustomMenuItem::new(
        TRAY_MENU_DEACTIVATE_AI_FEATURES.to_string(),
        format!("⏸️ Deactivate AI Features").as_str(),
    );

    let activate_ai_features = CustomMenuItem::new(
        TRAY_MENU_ACTIVATE_AI_FEATURES.to_string(),
        format!("▶️ Activate AI Features",).as_str(),
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
                .add_item(deactivate_ai_features)
                .add_item(version_label)
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(quit)
        } else {
            SystemTrayMenu::new()
                .add_item(activate_ai_features)
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
        SystemTrayEvent::MenuItemClick { id, .. } => {
            debug!(action = id.as_str(), "System tray action");
            match id.as_str() {
                TRAY_MENU_EXIT => on_quit(),
                TRAY_MENU_CHECK_AX_API => on_check_permissions(),
                TRAY_MENU_DEACTIVATE_AI_FEATURES => on_deactivate_ai_features(),
                TRAY_MENU_ACTIVATE_AI_FEATURES => on_activate_ai_features(),
                _ => {}
            }
        }

        _ => {}
    }
}

fn on_quit() {
    app_handle().exit(0);
}

fn on_check_permissions() {
    platform::macos::is_application_trusted_with_prompt();
}

fn on_deactivate_ai_features() {
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

    _ = app_handle().save_core_engine_state();
}

fn on_activate_ai_features() {
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

    _ = app_handle().save_core_engine_state();
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
