# AX Server Companion App (Rust) 

With this Rust-based companion app I am starting with an example app that connects to the Binance websocket API: [TMS Blog - Easily connect to Binance websocket streams with Rust](https://tms-dev-blog.com/easily-connect-to-binance-websocket-streams-with-rust/)


## Helper Stuff

```Rust
// let options: AppFocusState = Default::default();

// let new_event: Event = Event::AppFocusState(options);

// let new_message: AXMessage = AXMessage::Event(new_event);

// println!("{}", serde_json::to_string_pretty(&new_message).unwrap());

// let debug_connect_struct: AppFocusState = Default::default();
// let debug_new_request: Event = Event::AppFocusState(debug_connect_struct);
// let debug_new_message: Message = Message::Event(debug_new_request);
// let debug_new_ws_message: ws_message::WebsocketMessage<Message> =
//     ws_message::WebsocketMessage {
//         client: Uuid::new_v4(),
//         data: debug_new_message,
//     };

// let j = serde_json::to_string_pretty(&debug_new_ws_message).unwrap();

// // Print, write to a file, or send to an HTTP server.
// println!("{}", j);

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
```