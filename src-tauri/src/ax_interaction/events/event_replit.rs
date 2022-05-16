use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use super::models::editor::{
    EditorAppActivatedMessage, EditorAppClosedMessage, EditorAppCodeSelectedMessage,
    EditorAppDeactivatedMessage, EditorUIElementFocusedMessage, EditorWindowCreatedMessage,
    EditorWindowDestroyedMessage, EditorWindowMovedMessage, EditorWindowResizedMessage,
};

pub static AX_EVENT_REPLIT_CHANNEL: &str = "AXEventReplit";

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum AXEventReplit {
    EditorWindowCreated(EditorWindowCreatedMessage),
    EditorWindowDestroyed(EditorWindowDestroyedMessage),
    EditorWindowResized(EditorWindowResizedMessage),
    EditorWindowMoved(EditorWindowMovedMessage),
    EditorUIElementFocused(EditorUIElementFocusedMessage),
    EditorAppActivated(EditorAppActivatedMessage),
    EditorAppDeactivated(EditorAppDeactivatedMessage),
    EditorAppClosed(EditorAppClosedMessage),
    EditorAppCodeSelected(EditorAppCodeSelectedMessage),
    None,
}

impl fmt::Display for AXEventReplit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AXEventReplit::EditorWindowCreated(_) => write!(f, "EditorWindowCreated"),
            AXEventReplit::EditorWindowDestroyed(_) => write!(f, "EditorWindowDestroyed"),
            AXEventReplit::EditorWindowResized(_) => write!(f, "EditorWindowResized"),
            AXEventReplit::EditorWindowMoved(_) => write!(f, "EditorWindowMoved"),
            AXEventReplit::EditorUIElementFocused(_) => write!(f, "EditorUIElementFocused"),
            AXEventReplit::EditorAppActivated(_) => write!(f, "EditorAppActivated"),
            AXEventReplit::EditorAppDeactivated(_) => write!(f, "EditorAppDeactivated"),
            AXEventReplit::EditorAppClosed(_) => write!(f, "EditorClosed"),
            AXEventReplit::EditorAppCodeSelected(_) => write!(f, "EditorAppCodeSelected"),
            AXEventReplit::None => write!(f, "None"),
        }
    }
}

impl AXEventReplit {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = AX_EVENT_REPLIT_CHANNEL.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
