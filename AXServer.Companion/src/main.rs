// https://tms-dev-blog.com/easily-connect-to-binance-websocket-streams-with-rust/

use tungstenite::connect;
use url::Url;

use ax_messages::message::Message;
use ax_messages::types::{Event, Request, Response};
use ws_message::WebsocketMessage;

use ax_messages::models::AppFocusState;
use uuid::Uuid;

use crate::ax_messages::types::request::Connect;

mod ax_messages;
mod ws_message;

static AX_SERVER_URL: &str = "ws://127.0.0.1:8080";
fn main() {
    // let options: AppFocusState = Default::default();

    // let new_event: Event = Event::AppFocusState(options);

    // let new_message: AXMessage = AXMessage::Event(new_event);

    // println!("{}", serde_json::to_string_pretty(&new_message).unwrap());

    let debug_connect_struct: AppFocusState = Default::default();
    let debug_new_request: Event = Event::AppFocusState(debug_connect_struct);
    let debug_new_message: Message = Message::Event(debug_new_request);
    let debug_new_ws_message: ws_message::WebsocketMessage<Message> =
        ws_message::WebsocketMessage {
            client: Uuid::new_v4(),
            data: debug_new_message,
        };

    let j = serde_json::to_string_pretty(&debug_new_ws_message).unwrap();

    // // Print, write to a file, or send to an HTTP server.
    println!("{}", j);

    // let example_str = r#"{
    //   "client": "ee722b6b-0a03-43ed-93c8-b633b3c393a0",
    //   "data": {
    //     "Request": {
    //       "type": "Connect",
    //       "connect": true
    //     }
    //   }
    // }"#;

    // let deserialized: WebsocketMessage<Message> = serde_json::from_str(&example_str).unwrap();

    // println!("deserialized = {:#?}", deserialized);

    let ax_server_url = format!("{}/channel", AX_SERVER_URL);
    let (mut socket, response) =
        connect(Url::parse(&ax_server_url).unwrap()).expect("Can't connect.");

    println!("Connected to CodeAlpha AX Server.");
    println!("HTTP status code: {}", response.status());
    println!("Response headers:");
    for (ref header, ref header_value) in response.headers() {
        println!("- {}: {:?}", header, header_value);
    }

    let connect_struct: Connect = Connect { connect: true };
    let new_request: Request = Request::Connect(connect_struct);
    let new_message: Message = Message::Request(new_request);
    let new_ws_message: ws_message::WebsocketMessage<Message> = ws_message::WebsocketMessage {
        client: Uuid::new_v4(),
        data: new_message,
    };

    let serialized = serde_json::to_string_pretty(&new_ws_message).unwrap();

    socket
        .write_message(tungstenite::Message::Binary(serialized.into()))
        .unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message");

        print!("{}", msg);
    }
}
