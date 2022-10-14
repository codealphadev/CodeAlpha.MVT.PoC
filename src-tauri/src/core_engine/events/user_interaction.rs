use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{utils::messaging::ChannelList, app_handle};

use super::models::{
    CoreActivationStatusMessage, DismissSuggestionMessage, NodeAnnotationClickedMessage,
    PerformSuggestionMessage, UpdateSelectedSuggestionMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "bindings/user_interaction/")]
#[serde(tag = "event", content = "payload")]
pub enum EventUserInteraction {
    CoreActivationStatus(CoreActivationStatusMessage),
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
    }
}
