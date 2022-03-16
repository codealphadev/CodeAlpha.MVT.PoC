use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::accessibility_messages::types::{Event, Request, Response};
use super::accessibility_messages::Message;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketMessage<Message> {
    pub client: Uuid,
    pub data: Message,
}

impl WebsocketMessage<Message> {
    // Create WebsocketMessage of type Event from Type Event
    #[allow(dead_code)]
    pub fn from_event(event: Event, client_id: Uuid) -> WebsocketMessage<Message> {
        WebsocketMessage {
            client: client_id,
            data: Message::Event(event),
        }
    }

    pub fn from_request(request: Request, client_id: Uuid) -> WebsocketMessage<Message> {
        WebsocketMessage {
            client: client_id,
            data: Message::Request(request),
        }
    }

    #[allow(dead_code)]
    pub fn from_response(response: Response, client_id: Uuid) -> WebsocketMessage<Message> {
        WebsocketMessage {
            client: client_id,
            data: Message::Response(response),
        }
    }
}
