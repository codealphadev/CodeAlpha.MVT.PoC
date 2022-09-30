use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    utils::messaging::ChannelList,
    window_controls::{events::EventWindowControls, windows::MainWindow},
};

use super::handlers::{on_hide_app_window, on_show_app_window};

pub fn window_control_events_listener(main_window: &Arc<Mutex<MainWindow>>) {
    let main_window_move_copy = (main_window).clone();
    app_handle().listen_global(ChannelList::EventWindowControls.to_string(), move |msg| {
        let event_window_controls: EventWindowControls =
            serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match event_window_controls {
            EventWindowControls::TrackingAreaClicked(_) => {
                // Do Nothing here
            }
            EventWindowControls::TrackingAreaEntered(_) => {
                // Do Nothing here
            }
            EventWindowControls::TrackingAreaExited(_) => {
                // Do Nothing here
            }
            EventWindowControls::AppWindowHide(msg) => {
                on_hide_app_window(&main_window_move_copy, &msg);
            }
            EventWindowControls::AppWindowShow(msg) => {
                on_show_app_window(&main_window_move_copy, &msg);
            }
            EventWindowControls::DarkModeUpdate(_) => {
                // Do nothing here
            }
            EventWindowControls::TrackingAreaClickedOutside(_) => {
                // Do Nothing here
            }
            EventWindowControls::AppWindowUpdate(_) => {
                // Do Nothing here
            }
        }
    });
}
