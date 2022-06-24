use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::core_engine::CoreEngine;

pub fn register_listener(
    app_handle: &tauri::AppHandle,
    core_engine_props: &Arc<Mutex<CoreEngine>>,
) {
    let app_handle_move_copy = app_handle.clone();
    app_handle.listen_global("CHANNEL???", move |msg| {
        // let event_app_windows: TYPE??? = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        // match event_app_windows {
        //     _ => {}
        // }
    });
}
