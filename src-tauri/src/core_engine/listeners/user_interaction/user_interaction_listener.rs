use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    core_engine::{events::EventUserInteraction, CoreEngine},
    utils::messaging::ChannelList,
};

use super::handlers::on_core_activation_status_update;

pub fn user_interaction_listener(core_engine: &Arc<Mutex<CoreEngine>>) {
    app_handle().listen_global(ChannelList::EventUserInteractions.to_string(), {
        let core_engine = (core_engine).clone();
        move |msg| {
            let event_user_interaction: EventUserInteraction =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_user_interaction {
                EventUserInteraction::CoreActivationStatus(msg) => {
                    on_core_activation_status_update(&core_engine, &msg);
                }
                EventUserInteraction::None => {}
            }
        }
    });
}
