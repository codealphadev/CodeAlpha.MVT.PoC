use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ax_interaction::{CodeDocumentFrameProperties, ViewportProperties};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/macOS_specific/xcode/")]
pub struct ViewportPropertiesUpdateMessage {
    pub viewport_properties: Option<ViewportProperties>,
    pub code_document_frame_properties: Option<CodeDocumentFrameProperties>,
}
