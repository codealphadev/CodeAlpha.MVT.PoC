use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::{utils::messaging::ChannelList, window_controls::config::AppWindow};

use super::models::viewport::ViewportPropertiesUpdateMessage;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "bindings/macOS_specific/")]
pub enum EventViewport {
    XcodeViewportUpdate(ViewportPropertiesUpdateMessage),
}

impl fmt::Display for EventViewport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventViewport::XcodeViewportUpdate(_) => write!(f, "ViewportPropertiesUpdateMessage"),
        }
    }
}

impl EventViewport {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventViewport.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to CodeOverlay window
        _ = app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
