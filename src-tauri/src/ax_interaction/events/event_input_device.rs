use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::utils::messaging::ChannelList;

use super::models::input_device::{MouseClickMessage, MouseMovedMessage};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum EventInputDevice {
    MouseClick(MouseClickMessage),
    MouseMoved(MouseMovedMessage),
}

impl EventInputDevice {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventInputDevice.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
