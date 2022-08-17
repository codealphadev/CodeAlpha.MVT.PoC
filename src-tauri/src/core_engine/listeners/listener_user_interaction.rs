use std::sync::{Arc, Mutex};

use accessibility::AXUIElementAttributes;
use core_foundation::string::CFString;
use tauri::Manager;

use crate::{
    ax_interaction::{generate_axui_element_hash, get_textarea_uielement},
    core_engine::{
        events::{
            models::{CoreActivationStatusMessage, SearchQueryMessage},
            EventUserInteraction,
        },
        rules::RuleType,
        CoreEngine,
    },
    utils::messaging::ChannelList, app_handle,
};

pub fn register_listener_user_interactions(core_engine: &Arc<Mutex<CoreEngine>>) {
    let core_engine_move_copy = (core_engine).clone();
    app_handle().listen_global(ChannelList::EventUserInteractions.to_string(), move |msg| {
        let event_user_interaction: EventUserInteraction =
            serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match event_user_interaction {
            EventUserInteraction::CoreActivationStatus(msg) => {
                on_core_activation_status_update(&core_engine_move_copy, &msg);
            }
            EventUserInteraction::None => {}
        }
    });
}

fn on_core_activation_status_update(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    core_activation_status_msg: &CoreActivationStatusMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    core_engine.set_engine_active(core_activation_status_msg.engine_active);
}
