#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use websocket::websocket_client;

mod websocket;

static AX_SERVER_URL: &str = "ws://127.0.0.1:8080/channel";

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            tokio::spawn(async move {
                websocket_client::WebsocketClient::new(AX_SERVER_URL, handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
