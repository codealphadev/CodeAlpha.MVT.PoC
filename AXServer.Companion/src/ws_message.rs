use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketMessage<Message> {
    pub client: Uuid,
    pub data: Message,
}
