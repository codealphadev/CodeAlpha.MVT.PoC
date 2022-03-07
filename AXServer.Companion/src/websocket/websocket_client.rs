#![allow(unused_imports)]

use std::env;

use futures_util::{
    future, pin_mut,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};
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
    pub async fn new(url_string: &str) -> Self {
        let url = url::Url::parse(&url_string).expect("No valid URL path provided.");
        let ws_stream = Self::connect(&url).await;

        let client_id = Uuid::new_v4();

        Self::run(client_id, ws_stream).await;

        Self { url, client_id }
    }

    #[allow(dead_code)]
    pub async fn reconnect(&mut self, url: Url) {
        self.url = url;
        let ws_stream = Self::connect(&self.url).await;

        let client_id = Uuid::new_v4();

        Self::run(client_id, ws_stream).await;
    }

    async fn connect(url: &Url) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        return ws_stream;
    }

    async fn run(client_id: Uuid, ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>) {
        let (stream_write, stream_read) = ws_stream.split();

        let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();

        // Attempt connection to server
        let payload: accessibility_messages::models::Connect =
            accessibility_messages::models::Connect { connect: true };
        let ws_message = WebsocketMessage::from_request(
            accessibility_messages::types::Request::Connect(payload),
            client_id,
        );
        stdin_tx
            .unbounded_send(tungstenite::Message::binary(
                serde_json::to_vec(&ws_message).unwrap(),
            ))
            .unwrap();

        // Setup stdin stream to send messages to server
        tokio::spawn(Self::read_stdin(stdin_tx));

        // Setup stdout stream to receive messages from server
        let stdin_to_ws = stdin_rx.map(Ok).forward(stream_write);
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
            })
        };

        pin_mut!(stdin_to_ws, ws_to_stdout);
        future::select(stdin_to_ws, ws_to_stdout).await;
    }

    // Our helper method which will read data from stdin and send it along the
    // sender provided.
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
