use crate::window_controls::config::AppWindow;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppActivatedMessage {
    pub app_name: String,
    pub pid: u32,
    pub focused_app_window: Option<AppWindow>,
}
