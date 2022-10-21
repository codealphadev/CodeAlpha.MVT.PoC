use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{CustomMenuItem, Manager, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tracing::{debug, info};

use crate::{
    app_handle,
    app_state::{CoreEngineState, CoreEngineStateCache},
    core_engine::events::{
        models::{AiFeaturesStatusMessage, SwiftFormatOnCMDSMessage},
        EventUserInteraction,
    },
    platform,
};

use crate::app_state::AppHandleExtension;

const TRAY_MENU_ACTIVATE_AI_FEATURES: &str = "activate_ai_features";
const TRAY_MENU_DEACTIVATE_AI_FEATURES: &str = "deactivate_ai_features";
const TRAY_MENU_ACTIVATE_SWIFT_FORMAT_ON_CMD_S: &str = "activate_swift_format_on_cmd_s";
const TRAY_MENU_DEACTIVATE_SWIFT_FORMAT_ON_CMD_S: &str = "deactivate_swift_format_on_cmd_s";
const TRAY_MENU_EXIT: &str = "quit";
const TRAY_MENU_CHECK_AX_API: &str = "check_ax_api";
const TRAY_MENU_VERSION: &str = "version";

pub fn construct_system_tray_menu() -> SystemTrayMenu {
    let quit = CustomMenuItem::new(TRAY_MENU_EXIT.to_string(), "Quit");
    let check_ax_api = CustomMenuItem::new(TRAY_MENU_CHECK_AX_API.to_string(), "Settings...");

    if !platform::macos::is_application_trusted() {
        SystemTrayMenu::new()
            .add_item(check_ax_api)
            .add_item(version_menu_item())
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit)
    } else {
        SystemTrayMenu::new()
            .add_item(ai_features_menu_item())
            .add_item(swift_format_on_cmd_s_menu_item())
            .add_item(version_menu_item())
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(quit)
    }
}

fn version_menu_item() -> CustomMenuItem {
    let package_info = app_handle().package_info().clone();

    CustomMenuItem::new(
        TRAY_MENU_VERSION.to_string(),
        format!(
            "Version: {}.{}.{}",
            package_info.version.major, package_info.version.minor, package_info.version.patch
        )
        .as_str(),
    )
    .disabled()
}

fn ai_features_menu_item() -> CustomMenuItem {
    if check_ai_features() {
        CustomMenuItem::new(
            TRAY_MENU_DEACTIVATE_AI_FEATURES.to_string(),
            format!("Deactivate AI Features").as_str(),
        )
    } else {
        CustomMenuItem::new(
            TRAY_MENU_ACTIVATE_AI_FEATURES.to_string(),
            format!("Activate AI Features",).as_str(),
        )
    }
}

fn swift_format_on_cmd_s_menu_item() -> CustomMenuItem {
    if check_swift_format_on_cmd_s() {
        CustomMenuItem::new(
            TRAY_MENU_DEACTIVATE_SWIFT_FORMAT_ON_CMD_S.to_string(),
            format!("Deactivate SwiftFormat on ⌘+S").as_str(),
        )
    } else {
        CustomMenuItem::new(
            TRAY_MENU_ACTIVATE_SWIFT_FORMAT_ON_CMD_S.to_string(),
            format!("Activate SwiftFormat on ⌘+S",).as_str(),
        )
    }
}

fn check_ai_features() -> bool {
    if let Some(cache) = app_handle().try_state::<CoreEngineStateCache>() {
        cache.0.lock().ai_features_active
    } else {
        true
    }
}

fn check_swift_format_on_cmd_s() -> bool {
    if let Some(cache) = app_handle().try_state::<CoreEngineStateCache>() {
        cache.0.lock().swift_format_on_cmd_s
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
                TRAY_MENU_DEACTIVATE_SWIFT_FORMAT_ON_CMD_S => on_deactivate_swift_format_on_cmd_s(),
                TRAY_MENU_ACTIVATE_SWIFT_FORMAT_ON_CMD_S => on_activate_swift_format_on_cmd_s(),
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
        info!("User request: Deactivate AI Features");
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
        info!("User request: Activate AI Features");
        EventUserInteraction::AiFeaturesStatus(AiFeaturesStatusMessage {
            ai_features_active: check_ai_features(),
        })
        .publish_to_tauri();
    }

    _ = app_handle().save_core_engine_state();
}

fn on_deactivate_swift_format_on_cmd_s() {
    update_app_state_swift_format_on_cmd_s(false);

    if app_handle()
        .tray_handle()
        .set_menu(construct_system_tray_menu())
        .is_ok()
    {
        EventUserInteraction::SwiftFormatOnCMDS(SwiftFormatOnCMDSMessage {
            active: check_swift_format_on_cmd_s(),
        })
        .publish_to_tauri();
    }

    _ = app_handle().save_core_engine_state();
}

fn on_activate_swift_format_on_cmd_s() {
    update_app_state_swift_format_on_cmd_s(true);

    if app_handle()
        .tray_handle()
        .set_menu(construct_system_tray_menu())
        .is_ok()
    {
        EventUserInteraction::SwiftFormatOnCMDS(SwiftFormatOnCMDSMessage {
            active: check_swift_format_on_cmd_s(),
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

fn update_app_state_swift_format_on_cmd_s(swift_format_on_cmd_s: bool) {
    if let Some(cache) = app_handle().try_state::<CoreEngineStateCache>() {
        let current_core_engine_state = cache.0.lock().clone();
        *cache.0.lock() = CoreEngineState {
            swift_format_on_cmd_s,
            ..current_core_engine_state
        };
    } else {
        let cache = Arc::new(Mutex::new(CoreEngineState {
            swift_format_on_cmd_s,
            ..Default::default()
        }));
        app_handle().manage(CoreEngineStateCache(cache));
    };
}
