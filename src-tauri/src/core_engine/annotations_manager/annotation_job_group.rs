use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    core_engine::{events::AnnotationEvent, features::FeatureKind, EditorWindowUid},
    platform::macos::{get_code_document_frame_properties, get_visible_text_range, GetVia},
};

use super::{Annotation, AnnotationGroup, AnnotationJob, AnnotationJobTrait, AnnotationResult};

pub trait AnnotationJobGroupTrait {
    fn new(
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    ) -> Self;
    fn update_jobs(&mut self, jobs: Vec<AnnotationJob>);
    fn compute_annotations(&mut self);
    fn update_annotations(&mut self);

    fn get_annotation_group(&self) -> Option<AnnotationGroup>;
    fn get_annotation_job(&self, job_id: uuid::Uuid) -> Option<AnnotationJob>;
    fn get_annotation(&self, annotation_id: uuid::Uuid) -> Option<Annotation>;
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
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    ) -> Self {
        let id = uuid::Uuid::new_v4();
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

    fn update_jobs(&mut self, jobs: Vec<AnnotationJob>) {
        self.jobs = jobs
            .into_iter()
            .map(|job| (job.id(), job))
            .collect::<HashMap<uuid::Uuid, AnnotationJob>>();
    }

    fn compute_annotations(&mut self) {
        if let (Ok(visible_text_range), Ok(code_doc_props)) = (
            get_visible_text_range(GetVia::Current),
            get_code_document_frame_properties(&GetVia::Current),
        ) {
            for job in self.jobs.values_mut() {
                if let Ok(result) =
                    job.compute_bounds(&visible_text_range, &code_doc_props.dimensions.origin)
                {
                    self.results.insert(result.id, result);
                } else {
                    debug!(
                        "Failed to `compute_bounds` for job: {:?} for feature: {:?}",
                        job, self.feature
                    );
                }
            }

            self.publish_annotations();
        }
    }

    fn update_annotations(&mut self) {
        if let (Ok(visible_text_range), Ok(code_doc_props)) = (
            get_visible_text_range(GetVia::Current),
            get_code_document_frame_properties(&GetVia::Current),
        ) {
            for job in self.jobs.values_mut() {
                if let Ok(result) = job
                    .attempt_compute_bounds(&visible_text_range, &code_doc_props.dimensions.origin)
                {
                    self.results.insert(result.id, result);
                } else {
                    debug!(
                        "Failed `attempt_compute_bounds` for job: {:?} for feature: {:?}",
                        job, self.feature
                    );
                }
            }

            self.publish_annotations();
        }
    }

    fn get_annotation_group(&self) -> Option<AnnotationGroup> {
        let mut annotations = vec![];
        for job in self.jobs.values() {
            if let Some(annotation) = job.get_annotation() {
                annotations.push(annotation);
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

    fn get_annotation_job(&self, job_id: uuid::Uuid) -> Option<AnnotationJob> {
        self.jobs.get(&job_id).cloned()
    }

    fn get_annotation(&self, annotation_id: uuid::Uuid) -> Option<Annotation> {
        if let Some(annotation_job) = self.jobs.values().find(|job| job.id() == annotation_id) {
            annotation_job.get_annotation()
        } else {
            None
        }
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
