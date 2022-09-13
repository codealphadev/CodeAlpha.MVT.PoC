use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    platform::macos::{CodeDocumentFrameProperties, ViewportProperties},
    window_controls::config::AppWindow,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct UpdateAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
    pub viewport: ViewportProperties,
    pub code_document: CodeDocumentFrameProperties,
}
