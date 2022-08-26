use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::{
    ax_interaction::{get_code_document_frame_properties, get_viewport_properties, GetVia},
    utils::messaging::ChannelList,
    window_controls::config::AppWindow,
};

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

    pub fn new_xcode_viewport_update(get_via: &GetVia) -> Self {
        let viewport_properties = match get_viewport_properties(&get_via) {
            Ok(properties) => Some(properties),
            Err(_) => None,
        };
        let code_document_frame_properties = match get_code_document_frame_properties(&get_via) {
            Ok(properties) => Some(properties),
            Err(_) => None,
        };

        Self::XcodeViewportUpdate(ViewportPropertiesUpdateMessage {
            viewport_properties,
            code_document_frame_properties,
        })
    }
}
