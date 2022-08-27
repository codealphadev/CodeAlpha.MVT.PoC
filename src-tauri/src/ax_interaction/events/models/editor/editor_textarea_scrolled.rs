use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaScrolledMessage {
    pub window_uid: usize,
}
