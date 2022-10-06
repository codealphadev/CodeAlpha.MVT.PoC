use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{
    app_handle,
    core_engine::annotations_manager::{AnnotationGroup, AnnotationJobGroup},
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
    pub fn publish_to_tauri(&self) {
        let event_name = ChannelList::NodeAnnotationEvent.to_string();

        // Emit to frontend
        _ = app_handle().emit_to(
            &AppWindow::CodeOverlay.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum AnnotationManagerEvent {
    Add(Vec<AnnotationJobGroup>), // Appends the already present list of AnnotationJobGroup with the new ones.
    Update(Vec<AnnotationJobGroup>), // Updates existing AnnotationJobGroup with the new ones.
    Replace(Vec<AnnotationJobGroup>), // Replaces the already present list of AnnotationJobGroup with the new ones.
    Remove(Vec<uuid::Uuid>), // Removes the AnnotationJobGroup with the given IDs from the list.
}

impl AnnotationManagerEvent {
    pub fn publish_to_tauri(&self) {
        let event_name = ChannelList::NodeAnnotationEvent.to_string();

        // Emit to rust listeners
        app_handle().trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
