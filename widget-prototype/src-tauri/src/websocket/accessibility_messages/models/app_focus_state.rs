use serde::{Deserialize, Serialize};

use super::AppInfo;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppFocusState {
    pub previous_app: AppInfo,
    pub current_app: AppInfo,
}
