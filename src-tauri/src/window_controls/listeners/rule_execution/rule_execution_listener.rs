use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    core_engine::events::{models::NodeExplanationFetchedMessage, EventRuleExecutionState},
    utils::messaging::ChannelList,
    window_controls::{config::AppWindow, WindowManager},
};

pub fn rule_execution_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    let window_manager = (window_manager).clone();
    app_handle().listen_global(
        ChannelList::EventRuleExecutionState.to_string(),
        move |msg| {
            let event_rule_execution: EventRuleExecutionState =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_rule_execution {
                EventRuleExecutionState::NodeExplanationFetched(msg) => {
                    on_node_explanation_fetched(&window_manager, &msg);
                }
                _ => {
                    // Do Nothing here
                }
            }
        },
    );
}

fn on_node_explanation_fetched(
    window_manager: &Arc<Mutex<WindowManager>>,
    fetched_msg: &NodeExplanationFetchedMessage,
) -> Option<()> {
    let window_manager = window_manager.lock();

    window_manager.show_app_windows(
        vec![AppWindow::Explain],
        Some(fetched_msg.window_uid),
        fetched_msg.annotation_frame,
    );

    Some(())
}
