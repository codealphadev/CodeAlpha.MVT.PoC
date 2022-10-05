use std::collections::HashMap;

use crate::utils::geometry::LogicalFrame;

use super::annotation_job::{AnnotationJob, ViewportPositioning};

#[derive(thiserror::Error, Debug)]
pub enum AnnotationError {
    #[error("Something went wrong when executing the AnnotationManager.")]
    GenericError(#[source] anyhow::Error),
}

pub struct AnnotationResult {
    pub id: uuid::Uuid,
    pub position_relative_to_viewport: ViewportPositioning,
    pub bounds: Option<Vec<LogicalFrame>>,
    pub single_bounds: Option<LogicalFrame>,
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
