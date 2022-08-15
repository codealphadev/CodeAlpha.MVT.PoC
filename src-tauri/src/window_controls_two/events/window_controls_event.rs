use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::utils::messaging::ChannelList;

use super::models::{
    app_window::{HideAppWindowMessage, ShowAppWindowMessage},
    TrackingAreaClickedMessage, TrackingAreaEnteredMessage, TrackingAreaExitedMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum EventWindowControls {
    TrackingAreaClicked(TrackingAreaClickedMessage),
    TrackingAreaEntered(TrackingAreaEnteredMessage),
    TrackingAreaExited(TrackingAreaExitedMessage),
    AppWindowHide(HideAppWindowMessage),
    AppWindowShow(ShowAppWindowMessage),
}

impl EventWindowControls {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventWindowControls.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
