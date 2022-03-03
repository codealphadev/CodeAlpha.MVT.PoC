use serde::de;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XCodeFocusStatus {
    pub app_status: XCodeFocusStatusChange,
    pub editor_status: XCodeFocusStatusChange,
}
