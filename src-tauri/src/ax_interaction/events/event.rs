use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use super::models::{
    EditorAppActivatedMessage, EditorAppDeactivatedMessage, EditorUIElementFocusedMessage,
    EditorWindowCreatedMessage, EditorWindowDestroyedMessage, EditorWindowMovedMessage,
    EditorWindowResizedMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum AXEvent {
    EditorWindowCreated(EditorWindowCreatedMessage),
    EditorWindowDestroyed(EditorWindowDestroyedMessage),
    EditorWindowResized(EditorWindowResizedMessage),
    EditorWindowMoved(EditorWindowMovedMessage),
    EditorUIElementFocused(EditorUIElementFocusedMessage),
    EditorAppActivated(EditorAppActivatedMessage),
    EditorAppDeactivated(EditorAppDeactivatedMessage),
    None,
}

impl fmt::Display for AXEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AXEvent::EditorWindowCreated(_) => write!(f, "EditorWindowCreated"),
            AXEvent::EditorWindowDestroyed(_) => write!(f, "EditorWindowDestroyed"),
            AXEvent::EditorWindowResized(_) => write!(f, "EditorWindowResized"),
            AXEvent::EditorWindowMoved(_) => write!(f, "EditorWindowMoved"),
            AXEvent::EditorUIElementFocused(_) => write!(f, "EditorUIElementFocused"),
            AXEvent::EditorAppActivated(_) => write!(f, "EditorAppActivated"),
            AXEvent::EditorAppDeactivated(_) => write!(f, "EditorAppDeactivated"),
            AXEvent::None => write!(f, "None"),
        }
    }
}

static AX_EVENT_PREFIX: &str = "AXEvent";

impl AXEvent {
    pub fn publish_to_tauri(&self, app_handle: tauri::AppHandle) {
        let event_name = format!("{}-{}", AX_EVENT_PREFIX, self.to_string());

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }

    pub fn _tauri_event_name(event_type: AXEvent) -> String {
        format!("{}-{}", AX_EVENT_PREFIX, event_type.to_string())
    }
}
