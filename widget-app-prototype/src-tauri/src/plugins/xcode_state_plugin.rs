use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;

use crate::utils::xcode_twin::XCodeTwin;
use crate::websocket::accessibility_messages::models;

#[tauri::command]
fn register_again<R: Runtime>(_handle: tauri::AppHandle<R>, state: tauri::State<'_, XCodeTwin>) {
    // let client_id = Uuid::new_v4();
    // let payload: accessibility_messages::models::Connect =
    //     accessibility_messages::models::Connect { connect: true };
    // let ws_message = WebsocketMessage::from_request(
    //     accessibility_messages::types::Request::Connect(payload),
    //     client_id,
    // );
    // 2. Send client connection message through futures channel
    // let _result = state.send_generic_message(tungstenite::Message::binary(
    //     serde_json::to_vec(&ws_message).unwrap(),
    // ));

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

#[tauri::command]
fn get_app_focus_state(state: tauri::State<'_, XCodeTwin>) -> Option<models::AppFocusState> {
    state.get_state_global_app_focus()
}

#[tauri::command]
fn search_and_replace(search_str: String, replace_str: String, state: tauri::State<'_, XCodeTwin>) {
    if let Some(current_content) = state.get_state_xcode_editor_content() {
        let content_str = current_content.content;
        let content_str = content_str.replace(&search_str, &replace_str);

        state.update_xcode_editor_content(&content_str);
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("xcode-state-plugin")
        .invoke_handler(tauri::generate_handler![
            register_again,
            get_app_focus_state,
            search_and_replace
        ])
        .build()
}
