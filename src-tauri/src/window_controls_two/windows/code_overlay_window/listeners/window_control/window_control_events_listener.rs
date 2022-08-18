use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    utils::messaging::ChannelList,
    window_controls_two::{events::EventWindowControls, windows::CodeOverlayWindow},
};

use super::handlers::{on_hide_app_window, on_show_app_window};

pub fn window_control_events_listener(code_overlay_window: &Arc<Mutex<CodeOverlayWindow>>) {
    let code_overlay_window_move_copy = (code_overlay_window).clone();
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
                on_hide_app_window(&code_overlay_window_move_copy, &msg);
            }
            EventWindowControls::AppWindowShow(msg) => {
                on_show_app_window(&code_overlay_window_move_copy, &msg);
            }
            EventWindowControls::CodeOverlayDimensionsUpdate(_) => {
                // Do Nothing here
            }
        }
    });
}
