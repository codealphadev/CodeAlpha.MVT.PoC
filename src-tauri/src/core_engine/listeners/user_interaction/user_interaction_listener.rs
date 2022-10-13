use std::sync::Arc;

use crate::{
    app_handle,
    core_engine::{
        events::EventUserInteraction,
        features::{CoreEngineTrigger, UserCommand},
        CoreEngine,
    },
    utils::messaging::ChannelList,
};
use parking_lot::Mutex;
use tauri::Manager;
use tracing::debug;

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
                EventUserInteraction::NodeAnnotationClicked(msg) => {
                    _ = core_engine.lock().run_features(
                        msg.editor_window_uid,
                        &CoreEngineTrigger::OnUserCommand(UserCommand::NodeAnnotationClicked(msg)),
                    );
                }
                EventUserInteraction::PerformRefactoringOperation(msg) => {
                    debug!(?msg, "PerformRefactoringOperation request");
                    _ = core_engine.lock().run_features(
                        msg.editor_window_uid,
                        &CoreEngineTrigger::OnUserCommand(
                            UserCommand::PerformRefactoringOperation(msg),
                        ),
                    );
                }
                EventUserInteraction::DismissSuggestion(msg) => {
                    debug!(?msg, "DismissRefactoringSuggestion request");
                    _ = core_engine.lock().run_features(
                        msg.editor_window_uid,
                        &CoreEngineTrigger::OnUserCommand(UserCommand::DismissSuggestion(msg)),
                    );
                }
                EventUserInteraction::SelectSuggestion(msg) => {
                    debug!(?msg, "SelectRefactoringSuggestion request");
                    _ = core_engine.lock().run_features(
                        msg.editor_window_uid,
                        &CoreEngineTrigger::OnUserCommand(UserCommand::SelectSuggestion(msg)),
                    );
                }
                _ => {}
            }
        }
    });
}
