use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use super::models::editor::{
    EditorAppActivatedMessage, EditorAppClosedMessage, EditorAppDeactivatedMessage,
    EditorUIElementFocusedMessage, EditorWindowCreatedMessage, EditorWindowDestroyedMessage,
    EditorWindowMovedMessage, EditorWindowResizedMessage,
};

pub static AX_EVENT_XCODE_CHANNEL: &str = "AXEventXcode";

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum AXEventXcode {
    EditorWindowCreated(EditorWindowCreatedMessage),
    EditorWindowDestroyed(EditorWindowDestroyedMessage),
    EditorWindowResized(EditorWindowResizedMessage),
    EditorWindowMoved(EditorWindowMovedMessage),
    EditorUIElementFocused(EditorUIElementFocusedMessage),
    EditorAppActivated(EditorAppActivatedMessage),
    EditorAppDeactivated(EditorAppDeactivatedMessage),
    EditorClosed(EditorAppClosedMessage),
    None,
}

impl fmt::Display for AXEventXcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AXEventXcode::EditorWindowCreated(_) => write!(f, "EditorWindowCreated"),
            AXEventXcode::EditorWindowDestroyed(_) => write!(f, "EditorWindowDestroyed"),
            AXEventXcode::EditorWindowResized(_) => write!(f, "EditorWindowResized"),
            AXEventXcode::EditorWindowMoved(_) => write!(f, "EditorWindowMoved"),
            AXEventXcode::EditorUIElementFocused(_) => write!(f, "EditorUIElementFocused"),
            AXEventXcode::EditorAppActivated(_) => write!(f, "EditorAppActivated"),
            AXEventXcode::EditorAppDeactivated(_) => write!(f, "EditorAppDeactivated"),
            AXEventXcode::EditorClosed(_) => write!(f, "EditorClosed"),
            AXEventXcode::None => write!(f, "None"),
        }
    }
}

impl AXEventXcode {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = AX_EVENT_XCODE_CHANNEL.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
