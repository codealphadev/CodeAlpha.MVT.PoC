use serde::{Deserialize, Serialize};

use crate::window_controls::config::AppWindow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppWindowFocusedMessage {
    pub window: AppWindow,
}
