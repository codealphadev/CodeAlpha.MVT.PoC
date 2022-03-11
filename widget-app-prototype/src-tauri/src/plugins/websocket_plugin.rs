use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;
use tokio_tungstenite::tungstenite::{self};
use uuid::Uuid;

use crate::utils::xcode_twin::XCodeTwin;
use crate::websocket::accessibility_messages;
use crate::websocket::websocket_message::WebsocketMessage;

#[tauri::command]
fn register_again<R: Runtime>(_handle: tauri::AppHandle<R>, state: tauri::State<'_, XCodeTwin>) {
    let client_id = Uuid::new_v4();
    let payload: accessibility_messages::models::Connect =
        accessibility_messages::models::Connect { connect: true };
    let ws_message = WebsocketMessage::from_request(
        accessibility_messages::types::Request::Connect(payload),
        client_id,
    );
    // 2. Send client connection message through futures channel
    let _result = state.send_generic_message(tungstenite::Message::binary(
        serde_json::to_vec(&ws_message).unwrap(),
    ));

    if let Some(recent_message) = state.get_state_recent_message() {
        let print_str = serde_json::to_string(&recent_message).unwrap();

        print!("This works without multi threading! \n {} \n", print_str);
    }

    if let Some(recent_message) = state.get_state_xcode_editor_content() {
        let print_str = serde_json::to_string(&recent_message).unwrap();

        print!("This works without multi threading! \n {} \n", print_str);
    }

    if let Some(recent_message) = state.get_state_global_app_focus() {
        let print_str = serde_json::to_string(&recent_message).unwrap();

        print!("This works without multi threading! \n {} \n", print_str);
    }

    if let Some(recent_message) = state.get_state_xcode_focus_state() {
        let print_str = serde_json::to_string(&recent_message).unwrap();

        print!("This works without multi threading! \n {} \n", print_str);
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("awesome")
        .invoke_handler(tauri::generate_handler![register_again])
        .build()
}
