// https://tms-dev-blog.com/easily-connect-to-binance-websocket-streams-with-rust/

use tungstenite::connect;
use url::Url;
use uuid::Uuid;

use ax_messages::message::Message;
use ax_messages::types::{Event, Request, Response};

use ax_messages::models::{XCodeFocusElement, XCodeFocusStatusChange};
use ws_message::WebsocketMessage;

use crate::ax_messages::types::request::Connect;

mod ax_messages;
mod ws_message;

static AX_SERVER_URL: &str = "ws://127.0.0.1:8080";
fn main() {
    // Connect to the server
    let ax_server_url = format!("{}/channel", AX_SERVER_URL);
    let (mut socket, response) =
        connect(Url::parse(&ax_server_url).unwrap()).expect("Can't connect.");

    // Print connection response
    println!("Connected to CodeAlpha AX Server.");
    println!("HTTP status code: {}", response.status());
    println!("Response headers:");
    for (ref header, ref header_value) in response.headers() {
        println!("- {}: {:?}", header, header_value);
    }

    // Register as client at websocket server
    let connect_struct: Connect = Connect { connect: true };
    let new_request: Request = Request::Connect(connect_struct);
    let new_message: Message = Message::Request(new_request);
    let new_ws_message: ws_message::WebsocketMessage<Message> = ws_message::WebsocketMessage {
        client: Uuid::new_v4(),
        data: new_message,
    };

    let serialized_msg = serde_json::to_string_pretty(&new_ws_message).unwrap();

    socket
        .write_message(tungstenite::Message::Binary(serialized_msg.into()))
        .unwrap();

    // DEBUG
    // Register as client at websocket server
    // let asdasd = XCodeFocusStatusChange {
    //     focus_element_change: XCodeFocusElement::Editor,
    //     is_in_focus: true,
    // };
    // let check: Event = Event::XCodeFocusStatusChange(asdasd);
    // let duder: Message = Message::Event(check);
    // let bloeb: ws_message::WebsocketMessage<Message> = ws_message::WebsocketMessage {
    //     client: Uuid::new_v4(),
    //     data: duder,
    // };

    // let serialized2_msg = serde_json::to_string_pretty(&bloeb).unwrap();
    // print!("{}", serialized2_msg);

    // Start listening for received messages
    loop {
        let msg = socket.read_message().expect("Error reading message");

        let parsed_msg: WebsocketMessage<Message> = serde_json::from_str(&msg.to_string()).unwrap();

        print!("{:#?}", parsed_msg);
    }
}
