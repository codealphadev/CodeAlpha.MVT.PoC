mod ax_messages;
mod ws_message;

pub mod websocket_client;

static AX_SERVER_URL: &str = "ws://127.0.0.1:8080/channel";

#[tokio::main]
async fn main() {
    websocket_client::WebsocketClient::new(AX_SERVER_URL).await;
}
