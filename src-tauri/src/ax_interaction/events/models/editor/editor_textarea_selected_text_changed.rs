use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaSelectedTextChangedMessage {
    pub id: uuid::Uuid,
    pub ui_elem_hash: usize,
    pub index: usize,
    pub length: usize,
}
