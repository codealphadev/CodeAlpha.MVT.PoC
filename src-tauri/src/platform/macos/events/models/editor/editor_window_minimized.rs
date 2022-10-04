use serde::{Deserialize, Serialize};

use crate::core_engine::WindowUid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorWindowMinimizedMessage {
    pub window_uid: WindowUid,
}
