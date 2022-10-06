use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use crate::{
    core_engine::{features::FeatureKind, EditorWindowUid},
    utils::geometry::{LogicalFrame, LogicalPosition},
};

use super::annotation_job::{AnnotationJob, AnnotationKind, ViewportPositioning};

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

type AnnotationResults = HashMap<uuid::Uuid, AnnotationResult>;

trait AnnotationsManagerTrait {
    fn add_annotation_job(&mut self, job: AnnotationJob);
    fn update_annotation_job(&mut self, job: AnnotationJob);
    fn remove_annotation_job(&mut self, job_id: uuid::Uuid);
    fn get_annotation_job(&self, job_id: uuid::Uuid) -> Option<AnnotationJob>;
    fn compute_annotations(&mut self) -> AnnotationResults;
    fn scroll_to_annotation(&mut self, job_id: uuid::Uuid);
    fn publish_all(&mut self);
    fn publish_jobs_slice(&mut self, job_ids: &Vec<uuid::Uuid>);
}

pub struct AnnotationsManager {
    jobs: HashMap<uuid::Uuid, AnnotationJob>,
    results: HashMap<uuid::Uuid, AnnotationResult>,
}
