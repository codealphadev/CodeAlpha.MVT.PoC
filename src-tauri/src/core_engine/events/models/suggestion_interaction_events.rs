use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::EditorWindowUid;

#[derive(Clone, Debug, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, export_to = "bindings/user_interaction/")]
pub struct UpdateSelectedSuggestionMessage {
    pub id: Option<uuid::Uuid>,
    pub editor_window_uid: EditorWindowUid,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, export_to = "bindings/user_interaction/")]
pub struct PerformSuggestionMessage {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, export_to = "bindings/user_interaction/")]
pub struct DismissSuggestionMessage {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
}
