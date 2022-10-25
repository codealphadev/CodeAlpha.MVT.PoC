use std::sync::Arc;

use crate::{
    app_handle,
    core_engine::{
        events::EventUserInteraction,
        features::{CoreEngineTrigger, FeatureKind, UserCommand},
        CoreEngine,
    },
    utils::messaging::ChannelList,
};
use parking_lot::Mutex;
use tauri::Manager;
use tracing::info;

use super::handlers::{
    on_ai_features_activation_status_update, on_swift_format_on_cmd_s_activation_status_update,
};

pub fn user_interaction_listener(core_engine: &Arc<Mutex<CoreEngine>>) {
    app_handle().listen_global(ChannelList::EventUserInteractions.to_string(), {
        let core_engine = (core_engine).clone();
        move |msg| {
            let event_user_interaction: EventUserInteraction =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();
            match event_user_interaction {
                EventUserInteraction::AiFeaturesStatus(msg) => {
                    on_ai_features_activation_status_update(&core_engine, &msg);
                }
                EventUserInteraction::SwiftFormatOnCMDS(msg) => {
                    on_swift_format_on_cmd_s_activation_status_update(&core_engine, &msg);
                }
                EventUserInteraction::NodeAnnotationClicked(msg) => {
                    info!(
                        ?msg,
                        feature = FeatureKind::DocsGeneration.to_string(),
                        "User request: Node annotation clicked"
                    );

                    _ = core_engine.lock().run_features(
                        msg.editor_window_uid,
                        &CoreEngineTrigger::OnUserCommand(UserCommand::NodeAnnotationClicked(msg)),
                    );
                }
                EventUserInteraction::PerformSuggestion(msg) => {
                    info!(
                        ?msg,
                        feature = FeatureKind::ComplexityRefactoring.to_string(),
                        "User request: Perform suggestion"
                    );
                    _ = core_engine.lock().run_features(
                        msg.editor_window_uid,
                        &CoreEngineTrigger::OnUserCommand(UserCommand::PerformSuggestion(msg)),
                    );
                }
                EventUserInteraction::DismissSuggestion(msg) => {
                    info!(
                        ?msg,
                        feature = FeatureKind::ComplexityRefactoring.to_string(),
                        "User request: Dismiss suggestion"
                    );
                    _ = core_engine.lock().run_features(
                        msg.editor_window_uid,
                        &CoreEngineTrigger::OnUserCommand(UserCommand::DismissSuggestion(msg)),
                    );
                }
                EventUserInteraction::UpdateSelectedSuggestion(msg) => {
                    info!(
                        ?msg,
                        feature = FeatureKind::ComplexityRefactoring.to_string(),
                        "User request: Update selected suggestion"
                    );
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
