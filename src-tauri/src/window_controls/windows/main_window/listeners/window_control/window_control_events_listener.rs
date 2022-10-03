use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    utils::messaging::ChannelList,
    window_controls::{events::EventWindowControls, windows::MainWindow},
};

use super::handlers::{on_hide_app_window, on_show_app_window, on_update_app_window};

pub fn window_control_events_listener(main_window: &Arc<Mutex<MainWindow>>) {
    app_handle().listen_global(ChannelList::EventWindowControls.to_string(), {
        let main_window = main_window.clone();
        move |msg| {
            let event_window_controls: EventWindowControls =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_window_controls {
                EventWindowControls::AppWindowHide(msg) => {
                    on_hide_app_window(&main_window, &msg);
                }
                EventWindowControls::AppWindowShow(msg) => {
                    on_show_app_window(&main_window, &msg);
                }
                EventWindowControls::AppWindowUpdate(msg) => {
                    on_update_app_window(&main_window, &msg);
                }
                _ => {}
            }
        }
    });
}
