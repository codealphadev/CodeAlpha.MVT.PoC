use serde::{Deserialize, Serialize};

use super::XCodeFocusStatusChange;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XCodeFocusStatus {
    pub app_status: XCodeFocusStatusChange,
    pub editor_status: XCodeFocusStatusChange,
}
