use serde::{Deserialize, Serialize};

use crate::window_controls_two::config::AppWindow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HideAppWindowMessage {
    pub app_windows: Vec<AppWindow>,
}
