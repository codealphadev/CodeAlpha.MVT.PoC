use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    utils::messaging::ChannelList,
    window_controls_two::{events::EventWindowControls, widget_window::WidgetWindow},
};

pub fn window_control_events_listener(widget_window: &Arc<Mutex<WidgetWindow>>) {
    let widget_window_move_copy = (widget_window).clone();
    app_handle().listen_global(ChannelList::EventWindowControls.to_string(), move |msg| {
        let event_window_controls: EventWindowControls =
            serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match event_window_controls {
            EventWindowControls::TrackingAreaClicked(msg) => {
                // Do Nothing
            }
            EventWindowControls::TrackingAreaEntered(msg) => {
                // Do Nothing
            }
            EventWindowControls::TrackingAreaExited(msg) => {
                // Do Nothing
            }
            EventWindowControls::AppWindowHide(msg) => {
                // Do Nothing
            }
            EventWindowControls::AppWindowShow(msg) => {
                // Do Nothing
            }
        }
    });
}
