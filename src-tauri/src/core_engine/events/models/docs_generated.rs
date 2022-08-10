use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]
pub struct DocsGeneratedMessage {
    pub id: uuid::Uuid,
    pub text: String,
}
