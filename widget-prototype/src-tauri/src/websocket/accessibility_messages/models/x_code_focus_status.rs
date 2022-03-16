use serde::{Deserialize, Serialize};

use super::XCodeFocusStatusChange;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XCodeFocusStatus {
    pub app_status: XCodeFocusStatusChange,
    pub editor_status: XCodeFocusStatusChange,
}
