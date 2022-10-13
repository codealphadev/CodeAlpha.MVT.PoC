use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::models::{CoreActivationStatusMessage, NodeAnnotationClickedMessage, SuggestionMessage};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "bindings/user_interaction/")]
#[serde(tag = "event", content = "payload")]
pub enum EventUserInteraction {
    CoreActivationStatus(CoreActivationStatusMessage),
    PerformRefactoringOperation(SuggestionMessage),
    DismissSuggestion(SuggestionMessage),
    SelectSuggestion(SuggestionMessage),
    ToggleMainWindow(bool),
    NodeAnnotationClicked(NodeAnnotationClickedMessage),
}
