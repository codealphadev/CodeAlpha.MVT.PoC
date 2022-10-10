use anyhow::anyhow;
use log::error;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use ts_rs::TS;

use crate::{
    core_engine::{
        events::AnnotationManagerEvent, features::FeatureKind, EditorWindowUid, TextRange,
    },
    platform::macos::{
        get_bounds_for_TextRange, get_code_document_frame_properties,
        get_minimal_viewport_properties, get_visible_text_range, scroll_by_one_page, GetVia,
    },
    utils::geometry::{LogicalFrame, LogicalPosition},
};

use super::{
    listeners::{annotation_events::annotation_events_listener, xcode::xcode_listener},
    AnnotationJob, AnnotationJobGroup, AnnotationJobGroupTrait, AnnotationKind,
    ViewportPositioning,
};

static APROX_SCROLL_DURATION_PAGE_UP_DOWN_MS: u64 = 125;

#[derive(thiserror::Error, Debug)]
pub enum AnnotationError {
    #[error("Annotation of the given job uid does not exist.")]
    AnnotationNotFound,
    #[error("AnnotationGroup of the given uid does not exist.")]
    AnnotationJobGroupNotFound,
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
    pub char_index: usize,
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
    fn add_annotation_jobs_group(
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

    fn scroll_to_annotation(
        &mut self,
        group_id: uuid::Uuid,
        job_id: Option<uuid::Uuid>,
    ) -> Result<(), AnnotationError>;
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

    fn add_annotation_jobs_group(
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

    fn scroll_to_annotation(
        &mut self,
        group_id: uuid::Uuid,
        job_id: Option<uuid::Uuid>,
    ) -> Result<(), AnnotationError> {
        let annotation = self.get_annotation(group_id, job_id)?;

        tauri::async_runtime::spawn(async move {
            let visible_text_range = if let Ok(range) = get_visible_text_range(GetVia::Current) {
                range
            } else {
                return;
            };

            let viewport_positioning =
                Self::get_position_relative_to_viewport(annotation.char_index, &visible_text_range);

            match viewport_positioning {
                ViewportPositioning::Visible => {
                    _ = Self::scroll_procedure_if_annotation_within_visible_text_range(
                        &annotation,
                        group_id,
                    )
                    .await;
                }
                ViewportPositioning::InvisibleAbove => {
                    _ = Self::scroll_procedure_if_annotation_outside_visible_text_range(
                        ViewportPositioning::InvisibleAbove,
                        &annotation,
                        group_id,
                    )
                    .await;
                }
                ViewportPositioning::InvisibleBelow => {
                    _ = Self::scroll_procedure_if_annotation_outside_visible_text_range(
                        ViewportPositioning::InvisibleBelow,
                        &annotation,
                        group_id,
                    )
                    .await;
                }
            }
        });

        Ok(())
    }
}

impl AnnotationsManager {
    pub fn start_event_listeners(annotations_manager: &Arc<Mutex<Self>>) {
        annotation_events_listener(annotations_manager);
        xcode_listener(annotations_manager);
    }

    pub fn get_position_relative_to_viewport(
        char_index: usize,
        visible_text_range: &TextRange,
    ) -> ViewportPositioning {
        if char_index < visible_text_range.index {
            ViewportPositioning::InvisibleAbove
        } else if char_index > visible_text_range.index + visible_text_range.length {
            ViewportPositioning::InvisibleBelow
        } else {
            ViewportPositioning::Visible
        }
    }

    fn get_annotation(
        &mut self,
        group_id: uuid::Uuid,
        job_id: Option<uuid::Uuid>,
    ) -> Result<Annotation, AnnotationError> {
        let annotation_job_group = self
            .groups
            .get(&group_id)
            .ok_or(AnnotationError::AnnotationJobGroupNotFound)?;
        let annotation_group =
            annotation_job_group
                .get_annotation_group()
                .ok_or(AnnotationError::GenericError(
                    anyhow!("No AnnotationGroup computed yet").into(),
                ))?;

        let annotation = if let Some(job_id) = job_id {
            annotation_group
                .annotations
                .iter()
                .find(|a| a.id == job_id)
                .ok_or(AnnotationError::AnnotationNotFound)?
        } else {
            annotation_group
                .annotations
                .first()
                .ok_or(AnnotationError::AnnotationNotFound)?
        };

        Ok(annotation.clone())
    }

    pub fn get_annotation_rect_for_TextRange(
        text_range: &TextRange,
        editor_window_uid: Option<EditorWindowUid>,
    ) -> Option<LogicalFrame> {
        let get_via = match editor_window_uid {
            Some(editor_window_uid) => GetVia::Hash(editor_window_uid),
            None => GetVia::Current,
        };

        if let Ok(annotation_rect) = get_bounds_for_TextRange(&text_range, &get_via) {
            Some(annotation_rect)
        } else {
            None
        }
    }

    async fn scroll_procedure_if_annotation_outside_visible_text_range(
        positioning_relative_viewport: ViewportPositioning,
        annotation: &Annotation,
        group_id: uuid::Uuid,
    ) -> Result<(), AnnotationError> {
        match positioning_relative_viewport {
            ViewportPositioning::Visible => panic!("Should not happen"),
            ViewportPositioning::InvisibleAbove => {
                _ = scroll_by_one_page(true).await;
            }
            ViewportPositioning::InvisibleBelow => {
                _ = scroll_by_one_page(false).await;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(
            APROX_SCROLL_DURATION_PAGE_UP_DOWN_MS,
        ))
        .await;

        AnnotationManagerEvent::ScrollToAnnotationInGroup((group_id, Some(annotation.id)))
            .publish_to_tauri();

        Ok(())
    }

    async fn scroll_procedure_if_annotation_within_visible_text_range(
        annotation: &Annotation,
        group_id: uuid::Uuid,
    ) -> Result<(), AnnotationError> {
        let annotation_top_left =
            match annotation
                .shapes
                .first()
                .ok_or(AnnotationError::GenericError(
                    anyhow!("Annotation has no shapes").into(),
                ))? {
                AnnotationShape::Rectangle(frame) => frame.top_left(),
                AnnotationShape::Point(position) => *position,
            };

        match Self::annotation_visible_on_viewport(annotation_top_left)? {
            ViewportPositioning::Visible => {
                return Ok(());
            }
            ViewportPositioning::InvisibleAbove => {
                _ = scroll_by_one_page(true).await;
            }
            ViewportPositioning::InvisibleBelow => {
                _ = scroll_by_one_page(false).await;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(
            APROX_SCROLL_DURATION_PAGE_UP_DOWN_MS,
        ))
        .await;

        AnnotationManagerEvent::ScrollToAnnotationInGroup((group_id, Some(annotation.id)))
            .publish_to_tauri();

        Ok(())
    }

    fn annotation_visible_on_viewport(
        position_on_code_doc: LogicalPosition,
    ) -> Result<ViewportPositioning, AnnotationError> {
        let (viewport_props, code_doc_props) = get_minimal_viewport_properties(&GetVia::Current)
            .map_err(|e| AnnotationError::GenericError(e.into()))?;
        let global_position = position_on_code_doc.to_global(&code_doc_props.dimensions.origin);

        if global_position.y < viewport_props.dimensions.origin.y {
            Ok(ViewportPositioning::InvisibleAbove)
        } else if global_position.y
            > viewport_props.dimensions.origin.y + viewport_props.dimensions.size.height
        {
            Ok(ViewportPositioning::InvisibleBelow)
        } else {
            Ok(ViewportPositioning::Visible)
        }
    }
}
