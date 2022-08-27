use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaContentChangedMessage {
    pub window_uid: usize,
    pub pid: i32,
    pub content: String,
    pub file_path_as_str: Option<String>,
}
