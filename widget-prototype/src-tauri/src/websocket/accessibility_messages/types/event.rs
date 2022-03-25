use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

pub use super::super::models::{
    AppFocusState, XCodeEditorContent, XCodeFocusStatus, XCodeFocusStatusChange,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum Event {
    AppFocusState(AppFocusState),
    XCodeEditorContent(XCodeEditorContent),
    XCodeFocusStatus(XCodeFocusStatus),
    XCodeFocusStatusChange(XCodeFocusStatusChange),
    None,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::AppFocusState(_) => write!(f, "AppFocusState"),
            Event::XCodeEditorContent(_) => write!(f, "XCodeEditorContent"),
            Event::XCodeFocusStatus(_) => write!(f, "XCodeFocusStatus"),
            Event::XCodeFocusStatusChange(_) => write!(f, "XCodeFocusStatusChange"),
            Event::None => write!(f, "unknown"),
        }
    }
}

impl Event {
    pub fn publish_to_tauri(&self, app_handle: tauri::AppHandle) {
        let event_name = format!("StateEvent-{}", self.to_string());

        // Emit to frontend window listeners
        app_handle
            .emit_all(event_name.as_str(), self.clone())
            .unwrap();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
