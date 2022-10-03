use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub struct RemoveSuggestionMessage {
    pub id: Uuid,
}
