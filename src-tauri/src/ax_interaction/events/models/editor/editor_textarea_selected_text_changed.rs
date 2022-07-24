use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaSelectedTextChangedMessage {
    pub id: uuid::Uuid,
    pub index: usize,
    pub length: usize,
    pub selected_text: String,
    pub ui_elem_hash: usize,
}
