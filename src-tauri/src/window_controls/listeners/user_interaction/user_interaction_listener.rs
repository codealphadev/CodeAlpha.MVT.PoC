use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, core_engine::events::EventUserInteraction, utils::messaging::ChannelList,
    window_controls::WindowManager,
};

use super::handlers::on_core_activation_status_update;

pub fn user_interaction_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    let window_manager_move_copy = (window_manager).clone();
    app_handle().listen_global(ChannelList::EventUserInteractions.to_string(), move |msg| {
        let event_user_interaction: EventUserInteraction =
            serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match event_user_interaction {
            EventUserInteraction::CoreActivationStatus(msg) => {
                on_core_activation_status_update(&window_manager_move_copy, &msg);
            }
            EventUserInteraction::SearchQuery(_) => {
                // Do Nothing here - DEPRECATED, needs refactoring
            }
            EventUserInteraction::None => {
                // Do Nothing here - DEPRECATED, needs refactoring
            }
        }
    });
}
