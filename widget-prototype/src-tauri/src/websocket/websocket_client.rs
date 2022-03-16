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

pub async fn connect_to_ax_server(
    url: url::Url,
    client_id: Uuid,
    event_sink: futures_channel::mpsc::UnboundedSender<Message>,
    channel_sender: futures_channel::mpsc::UnboundedSender<Message>,
    channel_receiver: futures_channel::mpsc::UnboundedReceiver<Message>,
) {
    // 0. Establish connection to websocket server.
    // Returns a stream which we map to channels for sending and receiving messages.
    let (ws_stream, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    let (stream_write, stream_read) = ws_stream.split();

    // 1. Map channel receiver to websocket stream _write_ end.
    // This way, messages sent to the TX part of the channel from
    // anywhere in the program will be sent to the websocket stream
    // to later arrive in on the websocket server
    let send_to_ws_stream_connector = channel_receiver.map(Ok).forward(stream_write);

    // Attempt connection to server
    // 1. Construct client connection message
    let payload: accessibility_messages::models::Connect =
        accessibility_messages::models::Connect { connect: true };
    let ws_message = WebsocketMessage::from_request(
        accessibility_messages::types::Request::Connect(payload),
        client_id,
    );
    // 2. Send client connection message through futures channel
    let _result = channel_sender.unbounded_send(tungstenite::Message::binary(
        serde_json::to_vec(&ws_message).unwrap(),
    ));

    // Setup stdout stream to receive messages from server
    let ws_to_stdout = {
        stream_read.for_each(|message| async {
            let data = message.unwrap().into_text().unwrap();
            let parsed_msg: WebsocketMessage<accessibility_messages::Message> =
                serde_json::from_str(&data.to_string()).unwrap();

            // let print_str = serde_json::to_string(&parsed_msg).unwrap();
            // tokio::io::stdout()
            //     .write_all(&print_str.as_bytes())
            //     .await
            //     .unwrap();

            let _res = event_sink.unbounded_send(tungstenite::Message::binary(
                serde_json::to_vec(&parsed_msg).unwrap(),
            ));
        })
    };

    pin_mut!(send_to_ws_stream_connector, ws_to_stdout);
    future::select(send_to_ws_stream_connector, ws_to_stdout).await;
}
