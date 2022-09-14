use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{utils::messaging::ChannelList, window_controls::config::AppWindow};

use super::models::{
    NodeExplanationFetchedMessage, RemoveNodeAnnotationMessage, UpdateNodeAnnotationMessage,
};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(tag = "event", content = "payload")]
#[ts(export, export_to = "bindings/features/docs_generation/")]
pub enum EventDocsGeneration {
    UpdateNodeAnnotation(UpdateNodeAnnotationMessage),
    RemoveNodeAnnotation(RemoveNodeAnnotationMessage),
    NodeExplanationFetched(NodeExplanationFetchedMessage),
}

impl EventDocsGeneration {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventDocsGeneration.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to frontend
        _ = app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to frontend
        _ = app_handle.emit_to(
            &AppWindow::Explain.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
