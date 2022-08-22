use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utils::geometry::LogicalFrame;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]
pub struct CodeAnnotationMessage {
    pub id: uuid::Uuid,
    pub annotation_icon: Option<LogicalFrame>,
    pub annotation_codeblock: Option<LogicalFrame>,
}
