use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    core_engine::{events::AnnotationEvent, features::FeatureKind, EditorWindowUid, TextRange},
    utils::geometry::LogicalPosition,
};

use super::{AnnotationGroup, AnnotationJob, AnnotationJobTrait, AnnotationResult};

pub trait AnnotationJobGroupTrait {
    fn new(
        id: uuid::Uuid, // We require the caller to provide an id upfront, so that we can update the group later
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    ) -> Self;
    fn id(&self) -> uuid::Uuid;
    fn editor_window_uid(&self) -> EditorWindowUid;
    fn replace(&mut self, jobs: Vec<AnnotationJob>);
    fn compute_annotations(
        &mut self,
        visible_text_range: &TextRange,
        code_doc_origin: &LogicalPosition,
    );
    fn update_annotations(
        &mut self,
        visible_text_range: &TextRange,
        code_doc_origin: &LogicalPosition,
    );

    fn get_annotation_group(&self) -> Option<AnnotationGroup>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnotationJobGroup {
    id: uuid::Uuid,
    editor_window_uid: EditorWindowUid,
    feature: FeatureKind,
    group: Option<AnnotationGroup>,
    jobs: HashMap<uuid::Uuid, AnnotationJob>,
    results: HashMap<uuid::Uuid, AnnotationResult>,
}

impl AnnotationJobGroupTrait for AnnotationJobGroup {
    fn new(
        id: uuid::Uuid,
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    ) -> Self {
        let jobs = jobs
            .into_iter()
            .map(|job| (job.id(), job))
            .collect::<HashMap<uuid::Uuid, AnnotationJob>>();
        Self {
            id,
            feature,
            jobs,
            editor_window_uid,
            results: HashMap::new(),
            group: None,
        }
    }

    fn id(&self) -> uuid::Uuid {
        self.id
    }

    fn editor_window_uid(&self) -> EditorWindowUid {
        self.editor_window_uid
    }

    fn replace(&mut self, jobs: Vec<AnnotationJob>) {
        self.jobs = jobs
            .into_iter()
            .map(|job| (job.id(), job))
            .collect::<HashMap<uuid::Uuid, AnnotationJob>>();
        self.results = HashMap::new();
        self.group = None;
    }

    fn compute_annotations(
        &mut self,
        visible_text_range: &TextRange,
        code_doc_origin: &LogicalPosition,
    ) {
        for job in self.jobs.values_mut() {
            if let Ok(result) =
                job.compute_bounds(visible_text_range, code_doc_origin, self.editor_window_uid)
            {
                self.results.insert(result.id, result);
            } else {
                debug!(?job, feature = ?self.feature, "Failed to `compute_bounds`");
            }
        }

        self.publish_annotations();
    }

    fn update_annotations(
        &mut self,
        visible_text_range: &TextRange,
        code_doc_origin: &LogicalPosition,
    ) {
        for job in self.jobs.values_mut() {
            if let Ok(result) = job.compute_bounds_if_missing(
                visible_text_range,
                code_doc_origin,
                self.editor_window_uid,
            ) {
                self.results.insert(result.id, result);
            } else {
                debug!(?job, feature = ?self.feature, "Failed to `compute_bounds_if_missing`");
            }
        }

        self.publish_annotations();
    }

    fn get_annotation_group(&self) -> Option<AnnotationGroup> {
        let mut annotations = HashMap::new();
        for job in self.jobs.values() {
            if let Some(annotation) = job.get_annotation() {
                annotations.insert(annotation.id, annotation);
            } else {
                return None;
            }
        }

        Some(AnnotationGroup {
            id: self.id,
            feature: self.feature.clone(),
            annotations,
            editor_window_uid: self.editor_window_uid,
        })
    }
}

impl AnnotationJobGroup {
    fn publish_annotations(&mut self) {
        if let Some(annotation_group) = self.get_annotation_group() {
            if let Some(previous_group) = self.group.take() {
                // Case: new group is different from the previous group -> publish update
                if previous_group != annotation_group {
                    AnnotationEvent::UpdateAnnotationGroup(annotation_group.clone())
                        .publish_to_tauri();
                } else {
                    //  Case: new group is the same as the previous group -> no publish
                }
            } else {
                // Case: no previous group -> publish add
                AnnotationEvent::AddAnnotationGroup(annotation_group.clone()).publish_to_tauri();
            }

            self.group = Some(annotation_group);
        }
    }
}

impl Drop for AnnotationJobGroup {
    fn drop(&mut self) {
        AnnotationEvent::RemoveAnnotationGroup(self.id).publish_to_tauri();
    }
}
