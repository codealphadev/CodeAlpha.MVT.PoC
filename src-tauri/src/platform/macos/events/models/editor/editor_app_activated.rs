use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorAppActivatedMessage {
    pub editor_name: String,
    pub pid: u32,
}
