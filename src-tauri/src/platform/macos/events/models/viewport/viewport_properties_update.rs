use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::platform::macos::{CodeDocumentFrameProperties, ViewportProperties};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/macOS_specific/xcode/")]
pub struct ViewportPropertiesUpdateMessage {
    pub viewport_properties: ViewportProperties,
    pub code_document_frame_properties: CodeDocumentFrameProperties,
}
