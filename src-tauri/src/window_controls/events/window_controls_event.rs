use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{app_handle, utils::messaging::ChannelList, window_controls::TrackingEventSubscriber};

use super::models::{
    app_window::{HideAppWindowMessage, ShowAppWindowMessage, UpdateAppWindowMessage},
    dark_mode::DarkModeUpdateMessage,
    TrackingAreaClickedMessage, TrackingAreaClickedOutsideMessage, TrackingAreaEnteredMessage,
    TrackingAreaExitedMessage, TrackingAreaMouseOverMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
#[serde(tag = "event", content = "payload")]
pub enum EventWindowControls {
    TrackingAreaClicked(TrackingAreaClickedMessage),
    TrackingAreaClickedOutside(TrackingAreaClickedOutsideMessage),
    TrackingAreaEntered(TrackingAreaEnteredMessage),
    TrackingAreaExited(TrackingAreaExitedMessage),
    TrackingAreaMouseOver(TrackingAreaMouseOverMessage),
    AppWindowHide(HideAppWindowMessage),
    AppWindowShow(ShowAppWindowMessage),
    AppWindowUpdate(UpdateAppWindowMessage),
    DarkModeUpdate(DarkModeUpdateMessage),
}

impl EventWindowControls {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventWindowControls.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        let publish_to_frontend = match self {
            EventWindowControls::TrackingAreaClicked(_) => true,
            EventWindowControls::TrackingAreaEntered(_) => true,
            EventWindowControls::TrackingAreaExited(_) => true,
            EventWindowControls::TrackingAreaMouseOver(_) => true,
            EventWindowControls::TrackingAreaClickedOutside(_) => true,
            EventWindowControls::AppWindowHide(_) => true,
            EventWindowControls::AppWindowShow(_) => false,
            EventWindowControls::AppWindowUpdate(_) => false,
            EventWindowControls::DarkModeUpdate(_) => true,
        };

        // Emit to all windows
        if publish_to_frontend {
            _ = app_handle.emit_all(
                event_name.as_str(),
                Some(serde_json::to_string(self).unwrap()),
            );
        }
    }

    pub fn publish_tracking_area(&self, subscribers: &Vec<TrackingEventSubscriber>) {
        let event_name = ChannelList::EventWindowControls.to_string();

        for subscriber in subscribers {
            match subscriber {
                TrackingEventSubscriber::Backend => {
                    // Emit to rust listeners
                    app_handle().trigger_global(
                        event_name.as_str(),
                        Some(serde_json::to_string(self).unwrap()),
                    );
                }
                TrackingEventSubscriber::AppWindow(window_name) => {
                    _ = app_handle().emit_to(
                        &window_name.to_string(),
                        event_name.as_str(),
                        Some(serde_json::to_string(self).unwrap()),
                    );
                }
            }
        }
    }
}
