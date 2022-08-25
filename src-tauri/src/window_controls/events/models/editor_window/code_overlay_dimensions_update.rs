use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utils::geometry::LogicalFrame;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/code_overlay_dimensions_update/")]
pub struct CodeOverlayDimensionsUpdateMessage {
    pub code_viewport_rect: LogicalFrame,
    pub code_document_rect: LogicalFrame,
}
