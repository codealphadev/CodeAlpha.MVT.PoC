use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub bundle_id: String,
    pub name: String,
    pub pid: i32,
    pub is_finished_launching: bool,
}
