#![allow(unused_imports)]

use std::env;

use futures_util::{
    future, pin_mut,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tauri::{AppHandle, Manager, Runtime, Wry};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_tungstenite::{
    tungstenite::{self, Message},
    MaybeTlsStream, WebSocketStream,
};
use url::Url;
use uuid::Uuid;

// Project Imports
use super::accessibility_messages;
use super::websocket_message::WebsocketMessage;

pub struct WebsocketClient {
    pub url: Url,
    pub client_id: Uuid,
}

impl WebsocketClient {
    pub async fn new<R: Runtime>(
        url_string: &str,
        app_handle: &AppHandle<R>,
        future_sender: futures_channel::mpsc::UnboundedSender<Message>,
        future_receiver: futures_channel::mpsc::UnboundedReceiver<Message>,
    ) -> Self {
        let url = url::Url::parse(&url_string).expect("No valid URL path provided.");
        let client_id = Uuid::new_v4();

        let ws_stream = Self::connect(&url).await;
        let (stream_write, stream_read) = ws_stream.split();

        // Attempt connection to server
        // 1. Construct client connection message
        let payload: accessibility_messages::models::Connect =
            accessibility_messages::models::Connect { connect: true };
        let ws_message = WebsocketMessage::from_request(
            accessibility_messages::types::Request::Connect(payload),
            client_id,
        );
        // 2. Send client connection message through futures channel
        let _result = future_sender.unbounded_send(tungstenite::Message::binary(
            serde_json::to_vec(&ws_message).unwrap(),
        ));
        // 3. ... somehow ... bin websocket stream _sink_ to futures channel
        let stdin_to_ws = future_receiver.map(Ok).forward(stream_write);

        // Setup stdin stream to send messages to server
        // The following code is commented out because it blocks prints to stdout 🤔
        // tokio::spawn(Self::read_stdin(stdin_tx));

        // Setup stdout stream to receive messages from server
        let ws_to_stdout = {
            stream_read.for_each(|message| async {
                let data = message.unwrap().into_text().unwrap();
                let parsed_msg: WebsocketMessage<accessibility_messages::Message> =
                    serde_json::from_str(&data.to_string()).unwrap();

                // DEBUG
                let print_str = serde_json::to_string(&parsed_msg).unwrap();
                tokio::io::stdout()
                    .write_all(&print_str.as_bytes())
                    .await
                    .unwrap();

                app_handle.emit_all("ax-messages", parsed_msg).unwrap();
            })
        };

        pin_mut!(stdin_to_ws, ws_to_stdout);

        future::select(stdin_to_ws, ws_to_stdout).await;

        Self { url, client_id }
    }

    async fn connect(url: &Url) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        return ws_stream;
    }

    // Our helper method which will read data from stdin and send it along the
    // sender provided.
    #[allow(dead_code)]
    async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<tungstenite::Message>) {
        let mut stdin = tokio::io::stdin();
        loop {
            let mut buf = vec![0; 1024];
            let n = match stdin.read(&mut buf).await {
                Err(_) | Ok(0) => break,
                Ok(n) => n,
            };
            buf.truncate(n);
            tx.unbounded_send(tungstenite::Message::binary(buf))
                .unwrap();
        }
    }
}
