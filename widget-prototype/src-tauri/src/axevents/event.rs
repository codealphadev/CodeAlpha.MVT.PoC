use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

pub use super::models::{AppFocusState, XCodeFocusStatusChange};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum Event {
    AppFocusState(AppFocusState),
    XCodeFocusStatusChange(XCodeFocusStatusChange),
    None,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::AppFocusState(_) => write!(f, "AppFocusState"),
            Event::XCodeFocusStatusChange(_) => write!(f, "XCodeFocusStatusChange"),
            Event::None => write!(f, "unknown"),
        }
    }
}

impl Event {
    pub fn publish_to_tauri(&self, app_handle: tauri::AppHandle) {
        let event_name = format!("StateEvent-{}", self.to_string());

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
