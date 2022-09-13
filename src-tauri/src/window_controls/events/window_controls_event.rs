use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{utils::messaging::ChannelList, window_controls::config::AppWindow};

use super::models::{
    app_window::{HideAppWindowMessage, ShowAppWindowMessage},
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

        let publish_to_frontend;
        match self {
            EventWindowControls::TrackingAreaClicked(_) => publish_to_frontend = true,
            EventWindowControls::TrackingAreaEntered(_) => publish_to_frontend = true,
            EventWindowControls::TrackingAreaExited(_) => publish_to_frontend = true,
            EventWindowControls::AppWindowHide(_) => publish_to_frontend = false,
            EventWindowControls::AppWindowShow(_) => publish_to_frontend = false,
            EventWindowControls::DarkModeUpdate(_) => publish_to_frontend = true,
            EventWindowControls::TrackingAreaClickedOutside(_) => publish_to_frontend = true,
        }

        // Emit to CodeOverlay window
        if publish_to_frontend {
            _ = app_handle.emit_to(
                &AppWindow::CodeOverlay.to_string(),
                event_name.as_str(),
                Some(serde_json::to_string(self).unwrap()),
            );
        }
    }
}
