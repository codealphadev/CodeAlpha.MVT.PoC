use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaContentChanged {
    pub id: uuid::Uuid,
    pub content: String,
}
