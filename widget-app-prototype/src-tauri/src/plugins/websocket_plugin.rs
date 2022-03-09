use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};
use tokio_tungstenite::tungstenite::{self, Message};
use uuid::Uuid;

use crate::websocket::websocket_message::WebsocketMessage;
use crate::websocket::{accessibility_messages, websocket_client};

static AX_SERVER_URL: &str = "ws://127.0.0.1:8080/channel";

struct MyState {
    pub sender: futures_channel::mpsc::UnboundedSender<Message>,
}

#[tauri::command]
fn register_again<R: Runtime>(_handle: tauri::AppHandle<R>, state: tauri::State<'_, MyState>) {
    println!("register_again");
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

#[tauri::command]
fn my_custom_command() {
    println!("I was invoked from JS!");
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let (future_sender, future_receiver) = futures_channel::mpsc::unbounded();

    Builder::new("awesome")
        .invoke_handler(tauri::generate_handler![register_again, my_custom_command])
        .setup(|handle| {
            handle.manage(MyState {
                sender: future_sender.clone(),
            });

            let app_handle = handle.clone();
            tokio::spawn(async move {
                websocket_client::WebsocketClient::new(
                    AX_SERVER_URL,
                    &app_handle,
                    future_sender,
                    future_receiver,
                )
                .await;
            });

            Ok(())
        })
        .build()
}
