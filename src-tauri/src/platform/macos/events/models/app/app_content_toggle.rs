use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Copy, PartialEq)]
pub enum ContentWindowState {
    Active,
    Inactive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppContentActivationMessage {
    pub activation_state: ContentWindowState,
}
