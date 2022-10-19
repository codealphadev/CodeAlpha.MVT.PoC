use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{app_handle, utils::messaging::ChannelList};

use super::models::{
    AiFeaturesStatusMessage, DismissSuggestionMessage, NodeAnnotationClickedMessage,
    PerformSuggestionMessage, UpdateSelectedSuggestionMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "bindings/user_interaction/")]
#[serde(tag = "event", content = "payload")]
pub enum EventUserInteraction {
    AiFeaturesStatus(AiFeaturesStatusMessage),
    PerformSuggestion(PerformSuggestionMessage),
    DismissSuggestion(DismissSuggestionMessage),
    UpdateSelectedSuggestion(UpdateSelectedSuggestionMessage),
    ToggleMainWindow(bool),
    NodeAnnotationClicked(NodeAnnotationClickedMessage),
}

impl EventUserInteraction {
    pub fn publish_to_tauri(&self) {
        let event_name = ChannelList::EventUserInteractions.to_string();

        // Emit to rust listeners
        app_handle().trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to all windows
        _ = app_handle().emit_all(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
