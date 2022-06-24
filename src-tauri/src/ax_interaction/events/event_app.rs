use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::utils::messaging::ChannelList;

use super::models::app::{
    AppActivatedMessage, AppContentActivationMessage, AppDeactivatedMessage,
    AppUIElementFocusedMessage, AppWindowFocusedMessage, AppWindowMovedMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum AXEventApp {
    AppWindowFocused(AppWindowFocusedMessage),
    AppWindowMoved(AppWindowMovedMessage),
    AppUIElementFocused(AppUIElementFocusedMessage),
    AppContentActivationChange(AppContentActivationMessage),
    AppActivated(AppActivatedMessage),
    AppDeactivated(AppDeactivatedMessage),
    None,
}

impl fmt::Display for AXEventApp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AXEventApp::AppWindowMoved(_) => write!(f, "AppWindowMoved"),
            AXEventApp::AppUIElementFocused(_) => write!(f, "AppUIElementFocused"),
            AXEventApp::AppWindowFocused(_) => write!(f, "AppWindowFocused"),
            AXEventApp::AppContentActivationChange(_) => write!(f, "AppContentActivationChange"),
            AXEventApp::AppActivated(_) => write!(f, "AppActivated"),
            AXEventApp::AppDeactivated(_) => write!(f, "AppDeactivated"),
            AXEventApp::None => write!(f, "None"),
        }
    }
}

impl AXEventApp {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::AXEventApp.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
