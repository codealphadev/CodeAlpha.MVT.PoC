use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EditorWindowDestroyedMessage {
    pub id: uuid::Uuid,
}
