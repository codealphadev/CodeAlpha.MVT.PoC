use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};
use tokio_tungstenite::tungstenite::{self, Message};
use uuid::Uuid;

use super::websocket_plugin::MyState;

use crate::websocket::websocket_message::WebsocketMessage;
use crate::websocket::{accessibility_messages, websocket_client};

#[tauri::command]
fn another_command<R: Runtime>(_handle: tauri::AppHandle<R>, state: tauri::State<'_, MyState>) {
    let client_id = Uuid::new_v4();
    let payload: accessibility_messages::models::Connect =
        accessibility_messages::models::Connect { connect: true };
    let ws_message = WebsocketMessage::from_request(
        accessibility_messages::types::Request::Connect(payload),
        client_id,
    );
    // 2. Send client connection message through futures channel
    let _result = state.sender.unbounded_send(tungstenite::Message::binary(
        serde_json::to_vec(&ws_message).unwrap(),
    ));
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("test_plugin")
        .invoke_handler(tauri::generate_handler![another_command])
        .build()
}
