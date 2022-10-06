use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    app_handle,
    core_engine::{events::AnnotationEvent, features::FeatureKind},
};

use super::{Annotation, AnnotationJob};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub struct AnnotationGroup {
    pub id: uuid::Uuid,
    pub feature: FeatureKind,
    pub annotations: Vec<Annotation>,
}

impl Drop for AnnotationGroup {
    fn drop(&mut self) {
        AnnotationEvent::RemoveAnnotationGroup(self.id).publish_to_tauri(&app_handle());
    }
}

pub trait AnnotationJobGroupTrait {
    fn new(jobs: Vec<AnnotationJob>, feature: FeatureKind) -> Self;
}
