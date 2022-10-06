use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{
    core_engine::annotations_manager::{AnnotationGroup, AnnotationJob},
    utils::messaging::ChannelList,
    window_controls::config::AppWindow,
};

use super::models::{RemoveNodeAnnotationMessage, UpdateNodeAnnotationMessage};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(tag = "event", content = "payload")]
#[ts(export, export_to = "bindings/features/node_annotation/")]
pub enum AnnotationEvent {
    UpdateNodeAnnotation(UpdateNodeAnnotationMessage),
    RemoveNodeAnnotation(RemoveNodeAnnotationMessage),
    AddAnnotationGroup(AnnotationGroup),
    UpdateAnnotationGroup(AnnotationGroup),
    RemoveAnnotationGroup(uuid::Uuid),
}

impl AnnotationEvent {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::NodeAnnotationEvent.to_string();

        // Emit to frontend
        _ = app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum AnnotationManagerEvent {
    Add(Vec<AnnotationJob>), // Appends the already present list of AnnotationJob with the new ones.
    Update(Vec<AnnotationJob>), // Updates existing AnnotationJob with the new ones.
    Replace(Vec<AnnotationJob>), // Replaces the already present list of AnnotationJob with the new ones.
    Remove(Vec<uuid::Uuid>),     // Removes the AnnotationJob with the given IDs from the list.
}

impl AnnotationManagerEvent {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::NodeAnnotationEvent.to_string();

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
    }
}
