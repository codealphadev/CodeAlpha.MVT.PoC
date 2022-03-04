// https://tms-dev-blog.com/easily-connect-to-binance-websocket-streams-with-rust/

use tungstenite::connect;
use url::Url;

pub use models::{
    AppFocusState, AppInfo, Connect, Request, WebsocketMessage, XCodeEditorContent,
    XCodeFocusStatus, XCodeFocusStatusChange,
};

mod models;

static AX_SERVER_URL: &str = "ws://127.0.0.1:8080";
fn main() {
    let ax_server_url = format!("{}/channel", AX_SERVER_URL);
    let (mut socket, response) =
        connect(Url::parse(&ax_server_url).unwrap()).expect("Can't connect.");

    println!("Connected to CodeAlpha AX Server.");
    println!("HTTP status code: {}", response.status());
    println!("Response headers:");
    for (ref header, ref header_value) in response.headers() {
        println!("- {}: {:?}", header, header_value);
    }

    socket
        .write_message(tungstenite::Message::Binary(
            r#"{"client":"C13C2DA9-13FA-4BA6-A361-61488AC5B66A","data":{"connect":true}}"#.into(),
        ))
        .unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message");
        print!("{}", msg);
    }
}
