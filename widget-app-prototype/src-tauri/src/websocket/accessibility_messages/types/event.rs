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
        write!(f, "{:?}", self)
    }
}

impl Event {
    pub fn publish_to_tauri(&self, app_handle: Option<tauri::AppHandle>) {
        if let Some(handler) = app_handle.clone() {
            let event_name = format!("StateEvent-{}", self.to_string());

            handler.emit_all(event_name.as_str(), self.clone()).unwrap();
        }
    }
}
