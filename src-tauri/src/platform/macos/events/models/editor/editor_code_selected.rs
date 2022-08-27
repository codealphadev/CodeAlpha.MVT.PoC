use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorAppCodeSelectedMessage {
    pub code_selected: bool,
}
