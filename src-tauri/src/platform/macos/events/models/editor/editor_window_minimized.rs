use serde::{Deserialize, Serialize};

use crate::core_engine::EditorWindowUid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorWindowMinimizedMessage {
    pub window_uid: EditorWindowUid,
}
