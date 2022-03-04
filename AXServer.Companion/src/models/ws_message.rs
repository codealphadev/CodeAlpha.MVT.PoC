use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketMessage<T> {
    pub client: Uuid,
    pub data: T,
}
