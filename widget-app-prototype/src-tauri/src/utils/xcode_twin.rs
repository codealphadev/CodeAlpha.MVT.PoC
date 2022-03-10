use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::tungstenite::Message;

use crate::websocket::websocket_message::WebsocketMessage;
use crate::websocket::{accessibility_messages, websocket_client};

// How we do this:
// ======
// We create a "XCodeTwin" object that holds the most recently received state info from the
// accessibility server. So any Tauri command can retrieve this state info by interacting with
// the "XCodeTwin" object. Furthermore, this object is also responsible for emitting
// events to the frontend through Tauri. Messages that need to reach the Accessibility Server
// are also transferred through the "XCodeTwin" object.
pub struct XCodeTwin {
    accessibility_event_sender: futures_channel::mpsc::UnboundedSender<Message>,
    state_most_recent_message:
        Arc<tokio::sync::Mutex<Option<WebsocketMessage<accessibility_messages::Message>>>>,
}

impl XCodeTwin {
    pub fn new() -> Self {
        // Establish connection to Accessibility Server
        // ============================================

        // First, we create two sets of channels:
        // * The first TX and RX are being tied to the websocket server TcpStream
        // * The second TX and RX enable the struct we use to
        let (accessibility_event_sender, accessibility_event_receiver) =
            futures_channel::mpsc::unbounded();
        let (tauri_event_sender, mut tauri_event_receiver) = futures_channel::mpsc::unbounded();

        // Spawn connection to Accessibility Server from a separate thread
        let ax_sender = accessibility_event_sender.clone();
        tokio::spawn(async move {
            websocket_client::connect_to_ax_server(
                "ws://127.0.0.1:8080/channel",
                tauri_event_sender,
                ax_sender,
                accessibility_event_receiver,
            )
            .await;
        });

        // Connect XCodeTwin to state messages from Accessibility Server
        // ===========================================================

        // For now, only the most recent message is stored as "state".
        let recent_message: Option<WebsocketMessage<accessibility_messages::Message>> = None;
        let state_most_recent_message = Arc::new(tokio::sync::Mutex::new(recent_message));

        let c = Arc::clone(&state_most_recent_message);
        tokio::spawn(async move {
            loop {
                let result = tauri_event_receiver.try_next();
                if let Ok(Some(message)) = result {
                    let parsed_msg: WebsocketMessage<accessibility_messages::Message> =
                        serde_json::from_str(&message.to_string()).unwrap();

                    let mut val = c.lock().await;
                    *val = Some(parsed_msg.clone());

                    // DEBUG
                    let print_str = serde_json::to_string(&parsed_msg.clone()).unwrap();
                    tokio::io::stdout()
                        .write_all(&print_str.as_bytes())
                        .await
                        .unwrap();
                }
            }
        });

        Self {
            accessibility_event_sender,
            state_most_recent_message,
        }
    }

    pub fn send_generic_message(&self, message: Message) {
        let _result = self.accessibility_event_sender.unbounded_send(message);
    }

    pub fn get_state_recent_message(
        &self,
    ) -> Option<WebsocketMessage<accessibility_messages::Message>> {
        let check = Arc::clone(&self.state_most_recent_message);
        let val = check.try_lock().unwrap();

        return (*val).clone();
    }
}
