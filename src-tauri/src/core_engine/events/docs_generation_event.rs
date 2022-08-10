use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::{utils::messaging::ChannelList, window_controls::config::AppWindow};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum EventDocsGeneration {}

impl EventDocsGeneration {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventDocsGeneration.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to frontend
        _ = app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
