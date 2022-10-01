use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::utils::messaging::ChannelList;

use super::models::{
    app_window::{HideAppWindowMessage, ShowAppWindowMessage, UpdateAppWindowMessage},
    dark_mode::DarkModeUpdateMessage,
    TrackingAreaClickedMessage, TrackingAreaClickedOutsideMessage, TrackingAreaEnteredMessage,
    TrackingAreaExitedMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
#[serde(tag = "event", content = "payload")]
pub enum EventWindowControls {
    TrackingAreaClicked(TrackingAreaClickedMessage),
    TrackingAreaClickedOutside(TrackingAreaClickedOutsideMessage),
    TrackingAreaEntered(TrackingAreaEnteredMessage),
    TrackingAreaExited(TrackingAreaExitedMessage),
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
            EventWindowControls::AppWindowHide(_) => false,
            EventWindowControls::AppWindowShow(_) => false,
            EventWindowControls::AppWindowUpdate(_) => false,
            EventWindowControls::DarkModeUpdate(_) => true,
            EventWindowControls::TrackingAreaClickedOutside(_) => true,
        };

        // Emit to all windows
        if publish_to_frontend {
            _ = app_handle.emit_all(
                event_name.as_str(),
                Some(serde_json::to_string(self).unwrap()),
            );
        }
    }
}
