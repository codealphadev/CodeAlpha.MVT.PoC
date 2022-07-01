use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaScrolledMessage {
    pub id: uuid::Uuid,
    pub uielement_hash: usize,
}
