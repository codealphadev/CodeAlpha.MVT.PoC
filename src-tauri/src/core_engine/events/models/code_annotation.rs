use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::types::MatchRectangle;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]
pub struct CodeAnnotationMessage {
    pub id: uuid::Uuid,
    pub annotation_icon: Option<MatchRectangle>,
    pub annotation_codeblock: Option<MatchRectangle>,
}
