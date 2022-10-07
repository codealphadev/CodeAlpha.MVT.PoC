use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{utils::messaging::ChannelList, window_controls::config::AppWindow};

use super::models::ReplaceSuggestionsMessage;

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(tag = "event", content = "payload")]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub enum SuggestionEvent {
    ReplaceSuggestions(ReplaceSuggestionsMessage),
}

impl SuggestionEvent {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::SuggestionEvent.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to Main FE window
        _ = app_handle.emit_to(
            &AppWindow::Main.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to Widget FE window
        _ = app_handle.emit_to(
            &AppWindow::Widget.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
