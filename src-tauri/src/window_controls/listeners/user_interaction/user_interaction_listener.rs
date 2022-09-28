use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, core_engine::events::EventUserInteraction, utils::messaging::ChannelList,
    window_controls::WindowManager,
};

use super::handlers::{on_core_activation_status_update, on_main_window_toggle};

pub fn user_interaction_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    app_handle().listen_global(ChannelList::EventUserInteractions.to_string(), {
        let window_manager = (window_manager).clone();
        move |msg| {
            let event_user_interaction: EventUserInteraction =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_user_interaction {
                EventUserInteraction::CoreActivationStatus(msg) => {
                    on_core_activation_status_update(&window_manager, &msg);
                }
                EventUserInteraction::ToggleMainWindow(msg) => {
                    on_main_window_toggle(&window_manager, msg);
                }
            }
            EventUserInteraction::PerformRefactoringOperation(_) => (),
        }
    });
}
