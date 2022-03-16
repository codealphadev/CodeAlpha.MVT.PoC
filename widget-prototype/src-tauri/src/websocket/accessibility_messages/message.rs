use serde::{Deserialize, Serialize};

use super::types::{Event, Request, Response};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Message {
    Event(Event),
    Request(Request),
    Response(Response),
    None,
}
