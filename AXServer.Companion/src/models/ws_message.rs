use serde::de;
use serde::{Deserialize, Deserializer};

use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketMessage<T> {
    pub client: Uuid,
    pub data: T,
}
