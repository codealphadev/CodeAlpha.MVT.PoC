use serde::{Deserialize, Serialize};

use crate::window_controls::AppWindow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FocusedAppUIElement {
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppUIElementFocusedMessage {
    pub window: AppWindow,
    pub focused_ui_element: FocusedAppUIElement,
}
