use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::utils::messaging::ChannelList;

use super::models::{
    TrackingAreaClickedMessage, TrackingAreaEnteredMessage, TrackingAreaExitedMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
#[serde(tag = "event", content = "payload")]
pub enum EventWindowControls {
    TrackingAreaClicked(TrackingAreaClickedMessage),
    TrackingAreaEntered(TrackingAreaEnteredMessage),
    TrackingAreaExited(TrackingAreaExitedMessage),
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
