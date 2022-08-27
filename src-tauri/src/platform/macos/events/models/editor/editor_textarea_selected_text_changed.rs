use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaSelectedTextChangedMessage {
    pub window_uid: usize,
    pub index: usize,
    pub length: usize,
    pub selected_text: String,
}
