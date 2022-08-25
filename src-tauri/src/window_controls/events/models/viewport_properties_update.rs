use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utils::geometry::LogicalFrame;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct ViewportProperties {
    pub dimensions: LogicalFrame,
    pub annotation_section: LogicalFrame,
    pub code_section: LogicalFrame,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct CodeDocumentFrame {
    pub dimensions: LogicalFrame,
    pub text_offset: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct ViewportPropertiesUpdateMessage {
    pub viewport_properties: Option<ViewportProperties>,
    pub code_document_frame: Option<CodeDocumentFrame>,
}
