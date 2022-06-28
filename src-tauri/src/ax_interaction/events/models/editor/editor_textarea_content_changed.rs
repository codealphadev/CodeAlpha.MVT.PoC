use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaContentChangedMessage {
    pub id: uuid::Uuid,
    pub content: String,
    pub file_path_as_str: Option<String>,
}
