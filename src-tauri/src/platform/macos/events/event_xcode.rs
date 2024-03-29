use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::utils::messaging::ChannelList;

use super::models::editor::{
    EditorAppActivatedMessage, EditorAppClosedMessage, EditorAppDeactivatedMessage,
    EditorShortcutPressedMessage, EditorTextareaContentChangedMessage,
    EditorTextareaScrolledMessage, EditorTextareaSelectedTextChangedMessage,
    EditorTextareaZoomedMessage, EditorUIElementFocusedMessage, EditorWindowCreatedMessage,
    EditorWindowDestroyedMessage, EditorWindowMinimizedMessage, EditorWindowMovedMessage,
    EditorWindowResizedMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum AXEventXcode {
    EditorUIElementFocused(EditorUIElementFocusedMessage),
    EditorWindowCreated(EditorWindowCreatedMessage),
    EditorWindowDestroyed(EditorWindowDestroyedMessage),
    EditorWindowResized(EditorWindowResizedMessage),
    EditorWindowMoved(EditorWindowMovedMessage),
    EditorWindowMinimized(EditorWindowMinimizedMessage),
    EditorAppActivated(EditorAppActivatedMessage),
    EditorAppDeactivated(EditorAppDeactivatedMessage),
    EditorAppClosed(EditorAppClosedMessage),
    EditorTextareaScrolled(EditorTextareaScrolledMessage),
    EditorTextareaZoomed(EditorTextareaZoomedMessage),
    EditorTextareaContentChanged(EditorTextareaContentChangedMessage),
    EditorTextareaSelectedTextChanged(EditorTextareaSelectedTextChangedMessage),
    EditorShortcutPressed(EditorShortcutPressedMessage),
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
            AXEventXcode::EditorAppClosed(_) => write!(f, "EditorClosed"),
            AXEventXcode::EditorShortcutPressed(_) => write!(f, "EditorShortcutPressed"),
            AXEventXcode::EditorTextareaScrolled(_) => write!(f, "EditorTextareaScrolled"),
            AXEventXcode::EditorTextareaZoomed(_) => write!(f, "EditorTextareaZoomed"),
            AXEventXcode::EditorTextareaSelectedTextChanged(_) => {
                write!(f, "EditorTextareaSelectedTextChanged")
            }
            AXEventXcode::EditorTextareaContentChanged(_) => {
                write!(f, "EditorTextareaContentChanged")
            }
            AXEventXcode::EditorWindowMinimized(_) => write!(f, "EditorWindowMinimized"),
        }
    }
}

impl AXEventXcode {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::AXEventXcode.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
