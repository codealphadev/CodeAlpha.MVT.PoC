use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorTextareaScrollingFinishedMessage {
    pub window_uid: usize,
}
