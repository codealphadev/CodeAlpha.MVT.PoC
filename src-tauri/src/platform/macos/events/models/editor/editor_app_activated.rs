use serde::{Deserialize, Serialize};

use crate::core_engine::EditorWindowUid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorAppActivatedMessage {
    pub editor_name: String,
    pub pid: u32,
    pub window_uid: EditorWindowUid,
}
