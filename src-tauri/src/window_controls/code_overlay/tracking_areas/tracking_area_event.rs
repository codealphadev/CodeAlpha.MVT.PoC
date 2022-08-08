use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::utils::messaging::ChannelList;

use super::TrackingArea;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum EventTrackingArea {
    Add(Vec<TrackingArea>), // Appends the already present list of TrackingAreas with the new ones.
    Update(Vec<TrackingArea>), // Updates existing TrackingAreas with the new ones.
    Replace(Vec<TrackingArea>), // Replaces the already present list of TrackingAreas with the new ones.
    Remove(Vec<uuid::Uuid>),    // Removes the TrackingAreas with the given IDs from the list.
    Reset(),                    // Resets the list of TrackingAreas to an empty list.
}

impl EventTrackingArea {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventTrackingAreas.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
