use std::sync::Arc;

use crate::{
    app_handle, core_engine::events::EventUserInteraction, utils::messaging::ChannelList,
    window_controls::WindowManager,
};
use parking_lot::Mutex;
use tauri::Manager;
use tracing::info;

use super::handlers::on_main_window_toggle;

pub fn user_interaction_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    app_handle().listen_global(ChannelList::EventUserInteractions.to_string(), {
        let window_manager = (window_manager).clone();
        move |msg| {
            let event_user_interaction: EventUserInteraction =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_user_interaction {
                EventUserInteraction::ToggleMainWindow(msg) => {
                    info!(
                        ?msg,
                        feature = "ComplexityRefactoring",
                        "User request: Toggle main window"
                    );

                    on_main_window_toggle(&window_manager, msg);
                }
                _ => {}
            }
        }
    });
}
