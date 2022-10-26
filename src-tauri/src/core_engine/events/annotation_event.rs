use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{
    app_handle,
    core_engine::{
        annotations_manager::{AnnotationGroup, AnnotationJob, GetAnnotationInGroupVia},
        features::FeatureKind,
        EditorWindowUid,
    },
    utils::messaging::ChannelList,
    window_controls::config::AppWindow,
};

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(tag = "event", content = "payload")]
#[ts(export, export_to = "bindings/annotations/")]
pub enum AnnotationEvent {
    AddAnnotationGroup(AnnotationGroup),
    UpdateAnnotationGroup(AnnotationGroup),
    RemoveAnnotationGroup(uuid::Uuid),
}

impl AnnotationEvent {
    pub fn publish_to_tauri(&self) {
        let event_name = ChannelList::AnnotationEvent.to_string();

        let serialized_self = serde_json::to_string(self).ok();
        // tauri::async_runtime::spawn(async move {
        // Emit to frontend
        _ = app_handle().emit_to(
            &AppWindow::CodeOverlay.to_string(),
            event_name.as_str(),
            serialized_self.clone(),
        );
        // });
    }
}

type AnnotationGroupID = uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "payload")]
pub enum AnnotationManagerEvent {
    Add(
        (
            AnnotationGroupID,
            FeatureKind,
            Vec<AnnotationJob>,
            EditorWindowUid,
        ),
    ), // Creates a new annotation group for the given jobs; we don't let the caller submit a "JobsGroup" because of the JobsGroup's Drop implementation
    Upsert(
        (
            AnnotationGroupID,
            FeatureKind,
            Vec<AnnotationJob>,
            EditorWindowUid,
        ),
    ),
    Remove(AnnotationGroupID), // Removes the AnnotationJobGroup with the given IDs from the list.
    ScrollToAnnotationInGroup((AnnotationGroupID, GetAnnotationInGroupVia)),
}

impl AnnotationManagerEvent {
    pub fn publish_to_tauri(&self) {
        let event_name = ChannelList::AnnotationEvent.to_string();

        let serialized_self = serde_json::to_string(self).ok();
        // tauri::async_runtime::spawn(async move {
        app_handle().trigger_global(event_name.as_str(), serialized_self.clone());
        // });
    }
}
