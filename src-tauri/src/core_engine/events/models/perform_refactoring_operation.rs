use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::EditorWindowUid;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/user_interaction/")]
pub struct SuggestionMessage {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
}
