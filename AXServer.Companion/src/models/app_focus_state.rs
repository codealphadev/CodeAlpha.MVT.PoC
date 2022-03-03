use serde::de;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppFocusState {
    pub previous_app: AppInfo,
    pub current_app: AppInfo,
}
