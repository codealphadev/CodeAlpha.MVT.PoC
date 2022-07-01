use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaZoomedMessage {
    pub id: uuid::Uuid,
    pub uielement_hash: usize,
}
