use serde::Serialize;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::websocket::accessibility_messages::models;
use crate::websocket::websocket_message::WebsocketMessage;
use crate::websocket::{accessibility_messages, websocket_client};
use tokio_tungstenite::tungstenite;

// How we do this:
// ======
// We create a "XCodeTwin" object that holds the most recently received state info from the
// accessibility server. So any Tauri command can retrieve this state info by interacting with
// the "XCodeTwin" object. Furthermore, this object is also responsible for emitting
// events to the frontend through Tauri. Messages that need to reach the Accessibility Server
// are also transferred through the "XCodeTwin" object.
#[allow(dead_code)]
pub struct XCodeTwin {
    accessibility_event_sender: futures_channel::mpsc::UnboundedSender<Message>,

    state_most_recent_message:
        Arc<Mutex<Option<WebsocketMessage<accessibility_messages::Message>>>>,

    state_global_app_focus_state: Arc<Mutex<Option<models::AppFocusState>>>,
    state_xcode_editor_content: Arc<Mutex<Option<models::XCodeEditorContent>>>,
    state_xcode_editor_focus_state: Arc<Mutex<Option<models::XCodeFocusStatusChange>>>,
    state_xcode_app_focus_state: Arc<Mutex<Option<models::XCodeFocusStatusChange>>>,

    tauri_app_handle: Option<AppHandle>,

    client_id: uuid::Uuid,
}

#[allow(dead_code)]

impl XCodeTwin {
    pub fn new(url: url::Url) -> Self {
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
        let client_id = uuid::Uuid::new_v4();

        tokio::spawn(async move {
            websocket_client::connect_to_ax_server(
                url,
                client_id,
                tauri_event_sender,
                ax_sender,
                accessibility_event_receiver,
            )
            .await;
        });

        // Connect XCodeTwin to state messages from Accessibility Server
        // ===========================================================

        // Create Arc<Mutex>s for each state message
        let state_global_app_focus_state = Arc::new(Mutex::new(None));
        let state_xcode_editor_focus_state = Arc::new(Mutex::new(None));
        let state_xcode_app_focus_state = Arc::new(Mutex::new(None));
        let state_xcode_editor_content = Arc::new(Mutex::new(None));
        let state_most_recent_message = Arc::new(Mutex::new(None));

        // Clone all Arc<Mutex>s to be able to send them to a separate thread
        let app_focus_state = Arc::clone(&state_global_app_focus_state);
        let xcode_editor_focus_state = Arc::clone(&state_xcode_editor_focus_state);
        let xcode_app_focus_state = Arc::clone(&state_xcode_app_focus_state);
        let xcode_editor_content = Arc::clone(&state_xcode_editor_content);
        let most_recent_msg = Arc::clone(&state_most_recent_message);

        // Move AppHandle to thread to publish state messages
        let tauri_app_handle: Option<AppHandle> = None;
        let thread_app_handle = tauri_app_handle.clone();
        tokio::spawn(async move {
            loop {
                let result = tauri_event_receiver.try_next();
                if let Ok(Some(message)) = result {
                    let parsed_msg: WebsocketMessage<accessibility_messages::Message> =
                        serde_json::from_str(&message.to_string()).unwrap();

                    // Update most recently received message
                    Self::assign_msg(&most_recent_msg, &parsed_msg).await;

                    // // DEBUG
                    // let print_str = serde_json::to_string(&parsed_msg.clone()).unwrap();
                    // tokio::io::stdout()
                    //     .write_all(&print_str.as_bytes())
                    //     .await
                    //     .unwrap();

                    match parsed_msg.data {
                        accessibility_messages::Message::Event(data) => {
                            // Publish event to Tauri
                            data.publish_to_tauri(thread_app_handle.clone());

                            match data {
                                accessibility_messages::types::Event::AppFocusState(payload) => {
                                    Self::assign_msg(&app_focus_state, &payload).await;
                                }
                                accessibility_messages::types::Event::XCodeEditorContent(
                                    payload,
                                ) => {
                                    Self::assign_msg(&xcode_editor_content, &payload).await;
                                }
                                accessibility_messages::types::Event::XCodeFocusStatus(payload) => {
                                    Self::assign_msg(&xcode_app_focus_state, &payload.app_status)
                                        .await;
                                    Self::assign_msg(
                                        &xcode_editor_focus_state,
                                        &payload.editor_status,
                                    )
                                    .await;
                                }
                                accessibility_messages::types::Event::XCodeFocusStatusChange(
                                    payload,
                                ) => match payload.focus_element_change {
                                    accessibility_messages::models::XCodeFocusElement::App => {
                                        Self::assign_msg(&xcode_app_focus_state, &payload).await;
                                    }
                                    accessibility_messages::models::XCodeFocusElement::Editor => {
                                        Self::assign_msg(&xcode_editor_focus_state, &payload).await;
                                    }
                                },
                                accessibility_messages::types::Event::None => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        Self {
            accessibility_event_sender,
            state_most_recent_message,
            state_global_app_focus_state,
            state_xcode_editor_content,
            state_xcode_editor_focus_state,
            state_xcode_app_focus_state,
            tauri_app_handle,
            client_id,
        }
    }

    async fn assign_msg<T: Clone + Serialize>(old_val: &Arc<Mutex<Option<T>>>, new_val: &T) {
        let mut locked_val = old_val.lock().await;
        *locked_val = Some(new_val.clone());
    }

    fn send_generic_message(&self, message: Message) {
        let _result = self.accessibility_event_sender.unbounded_send(message);
    }

    // Reconnect happens by resetting the XCodeTwin -> removes all previous state
    pub fn reconnect(&self, url_path: &str, port: &str) -> Self {
        let url =
            url::Url::parse(&format!("{}{}", url_path, port)).expect("No valid URL path provided.");
        Self::new(url)
    }

    pub fn get_state_recent_message(
        &self,
    ) -> Option<WebsocketMessage<accessibility_messages::Message>> {
        let arc_handle = Arc::clone(&self.state_most_recent_message);
        let val = arc_handle.try_lock().unwrap();
        return (*val).clone();
    }

    pub fn get_state_xcode_editor_content(&self) -> Option<models::XCodeEditorContent> {
        let arc_handle = Arc::clone(&self.state_xcode_editor_content);
        let val = arc_handle.try_lock().unwrap();
        return (*val).clone();
    }

    pub fn get_state_global_app_focus(&self) -> Option<models::AppFocusState> {
        let arc_handle = Arc::clone(&self.state_global_app_focus_state);
        let val = arc_handle.try_lock().unwrap();
        return (*val).clone();
    }

    pub fn get_state_xcode_focus_state(&self) -> Option<models::XCodeFocusStatus> {
        let arc_handle_1 = Arc::clone(&self.state_xcode_editor_focus_state);
        let arc_handle_2 = Arc::clone(&self.state_xcode_app_focus_state);
        let val1 = arc_handle_1.try_lock().unwrap();
        let val2 = arc_handle_2.try_lock().unwrap();

        if (*val1).is_some() && (*val2).is_some() {
            return Some(models::XCodeFocusStatus {
                editor_status: (*val1).clone().unwrap(),
                app_status: (*val2).clone().unwrap(),
            });
        } else {
            return None;
        }
    }

    pub fn update_xcode_editor_content(&self, new_content: &str) {
        let content_update = models::XCodeEditorContent {
            content: new_content.to_string(),
            file_extension: "".to_string(),
            file_name: "".to_string(),
            file_path: "".to_string(),
        };

        let ws_message = WebsocketMessage::from_request(
            accessibility_messages::types::Request::UpdateXCodeEditorContent(content_update),
            self.client_id,
        );

        self.send_generic_message(tungstenite::Message::binary(
            serde_json::to_vec(&ws_message).unwrap(),
        ));
    }
}
