use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::window_controls::config::AppWindow;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct HideAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
}
