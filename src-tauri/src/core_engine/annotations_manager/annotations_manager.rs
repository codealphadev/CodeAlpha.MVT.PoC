use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use ts_rs::TS;

use crate::{
    core_engine::{features::FeatureKind, EditorWindowUid},
    platform::macos::{get_code_document_frame_properties, get_visible_text_range, GetVia},
    utils::geometry::{LogicalFrame, LogicalPosition},
};

use super::{
    listeners::{annotation_events::annotation_events_listener, xcode::xcode_listener},
    AnnotationJob, AnnotationJobGroup, AnnotationJobGroupTrait, AnnotationKind,
    ViewportPositioning,
};

#[derive(thiserror::Error, Debug)]
pub enum AnnotationError {
    #[error("Something went wrong when executing the AnnotationManager.")]
    GenericError(#[source] anyhow::Error),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub enum AnnotationShape {
    Rectangle(LogicalFrame),
    Point(LogicalPosition),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub struct Annotation {
    pub id: uuid::Uuid,
    pub kind: AnnotationKind,
    pub position_relative_to_viewport: ViewportPositioning,
    pub shapes: Vec<AnnotationShape>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub struct AnnotationGroup {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
    pub feature: FeatureKind,
    pub annotations: Vec<Annotation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnotationResult {
    pub id: uuid::Uuid,
    pub position_relative_to_viewport: ViewportPositioning,
    pub bounds: Option<Vec<LogicalFrame>>,
}

pub trait AnnotationsManagerTrait {
    fn new() -> Self;
    fn add_annotation_jobs(
        &mut self,
        group_id: uuid::Uuid,
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    );
    fn replace_annotation_job_group(&mut self, group_id: uuid::Uuid, jobs: Vec<AnnotationJob>);
    fn recompute_annotations(&mut self, editor_window_uid: EditorWindowUid);
    fn update_annotations(&mut self, editor_window_uid: EditorWindowUid);

    fn remove_annotation_job_group(&mut self, group_id: uuid::Uuid);
    fn remove_annotation_job_group_of_editor_window(&mut self, editor_window_uid: EditorWindowUid);
    fn reset(&mut self);

    fn scroll_to_annotation(&mut self, group_id: uuid::Uuid);
}

pub struct AnnotationsManager {
    groups: HashMap<uuid::Uuid, AnnotationJobGroup>,
}

impl AnnotationsManagerTrait for AnnotationsManager {
    fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    fn add_annotation_jobs(
        &mut self,
        group_id: uuid::Uuid,
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    ) {
        self.groups.insert(
            group_id,
            AnnotationJobGroup::new(group_id, feature, jobs, editor_window_uid),
        );
        if let (Ok(visible_text_range), Ok(code_doc_props)) = (
            get_visible_text_range(GetVia::Hash(editor_window_uid)),
            get_code_document_frame_properties(&GetVia::Hash(editor_window_uid)),
        ) {
            self.groups
                .get_mut(&group_id)
                .unwrap() // Unwrap safe here because we just inserted the group
                .compute_annotations(&visible_text_range, &code_doc_props.dimensions.origin);
        }
    }

    fn replace_annotation_job_group(&mut self, group_id: uuid::Uuid, jobs: Vec<AnnotationJob>) {
        if let Some(group) = self.groups.get_mut(&group_id) {
            if let (Ok(visible_text_range), Ok(code_doc_props)) = (
                get_visible_text_range(GetVia::Hash(group.editor_window_uid())),
                get_code_document_frame_properties(&GetVia::Hash(group.editor_window_uid())),
            ) {
                group.replace(jobs);
                group.compute_annotations(&visible_text_range, &code_doc_props.dimensions.origin);
            }
        }
    }

    fn recompute_annotations(&mut self, editor_window_uid: EditorWindowUid) {
        if let (Ok(visible_text_range), Ok(code_doc_props)) = (
            get_visible_text_range(GetVia::Hash(editor_window_uid)),
            get_code_document_frame_properties(&GetVia::Hash(editor_window_uid)),
        ) {
            for group in self.groups.values_mut() {
                if group.editor_window_uid() == editor_window_uid {
                    group.compute_annotations(
                        &visible_text_range,
                        &code_doc_props.dimensions.origin,
                    );
                }
            }
        }
    }

    fn update_annotations(&mut self, editor_window_uid: EditorWindowUid) {
        if let (Ok(visible_text_range), Ok(code_doc_props)) = (
            get_visible_text_range(GetVia::Hash(editor_window_uid)),
            get_code_document_frame_properties(&GetVia::Hash(editor_window_uid)),
        ) {
            for group in self.groups.values_mut() {
                if group.editor_window_uid() == editor_window_uid {
                    group
                        .update_annotations(&visible_text_range, &code_doc_props.dimensions.origin);
                }
            }
        }
    }

    fn remove_annotation_job_group(&mut self, group_id: uuid::Uuid) {
        self.groups.remove(&group_id);
    }

    fn remove_annotation_job_group_of_editor_window(&mut self, editor_window_uid: EditorWindowUid) {
        self.groups
            .retain(|_, group| group.editor_window_uid() != editor_window_uid);
    }

    fn reset(&mut self) {
        self.groups.clear();
    }

    fn scroll_to_annotation(&mut self, _group_id: uuid::Uuid) {
        todo!()
    }
}

impl AnnotationsManager {
    pub fn start_event_listeners(annotations_manager: &Arc<Mutex<Self>>) {
        annotation_events_listener(annotations_manager);
        xcode_listener(annotations_manager);
    }
}
