use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
