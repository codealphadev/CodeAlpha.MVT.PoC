use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/dark_mode_update/")]
pub struct DarkModeUpdateMessage {
    pub dark_mode: bool,
}
