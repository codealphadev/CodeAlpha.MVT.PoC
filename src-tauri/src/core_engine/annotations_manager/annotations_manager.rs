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
        get_bounds_for_TextRange, get_minimal_viewport_properties, get_visible_text_range,
        scroll_by_one_page, GetVia, XcodeError,
    },
    utils::geometry::{LogicalFrame, LogicalPosition},
};

use super::{
    listeners::{annotation_events::annotation_events_listener, xcode::xcode_listener},
    AnnotationJob, AnnotationJobGroup, AnnotationJobGroupTrait, AnnotationKind,
    VisibleTextRangePositioning,
};

static APPROX_SCROLL_DURATION_PAGE_UP_DOWN_MS: u64 = 125;

#[derive(thiserror::Error, Debug)]
pub enum AnnotationError {
    #[error("Annotation of the given job uid does not exist.")]
    AnnotationNotFound,
    #[error("AnnotationGroup of the given uid does not exist.")]
    AnnotationJobGroupNotFound,
    #[error("Something went wrong when executing the AnnotationManager.")]
    GenericError(#[source] anyhow::Error),
}

impl From<XcodeError> for AnnotationError {
    fn from(cause: XcodeError) -> Self {
        AnnotationError::GenericError(cause.into())
    }
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
    pub position_relative_to_viewport: VisibleTextRangePositioning,
    pub shapes: Vec<AnnotationShape>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub struct AnnotationGroup {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid,
    pub feature: FeatureKind,
    pub annotations: HashMap<uuid::Uuid, Annotation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnotationResult {
    pub id: uuid::Uuid,
    pub position_relative_to_viewport: VisibleTextRangePositioning,
    pub bounds: Option<Vec<LogicalFrame>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GetAnnotationInGroupVia {
    Id(uuid::Uuid),
    Kind(AnnotationKind),
}
pub enum ViewportPositioning {
    Visible,
    InvisibleAbove,
    InvisibleBelow,
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

    fn upsert_annotation_job_group(
        &mut self,
        group_id: uuid::Uuid,
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    );

    fn recompute_annotations(&mut self, editor_window_uid: EditorWindowUid);
    fn update_annotations(&mut self, editor_window_uid: EditorWindowUid);

    fn remove_annotation_job_group(&mut self, group_id: uuid::Uuid);
    fn remove_annotation_job_group_of_editor_window(&mut self, editor_window_uid: EditorWindowUid);
    fn reset(&mut self);

    fn scroll_to_annotation(
        &mut self,
        group_id: uuid::Uuid,
        get_job_via: GetAnnotationInGroupVia,
    ) -> Result<(), AnnotationError>;
}

pub struct AnnotationsManager {
    groups: HashMap<uuid::Uuid, AnnotationJobGroup>,

    // Because scrolling _takes time_ we want to allow to interrupt the scrolling process if the user wants to scroll to another annotation.
    scroll_to_annotation_job_id: Arc<Mutex<Option<uuid::Uuid>>>,
}

impl AnnotationsManagerTrait for AnnotationsManager {
    fn new() -> Self {
        Self {
            groups: HashMap::new(),
            scroll_to_annotation_job_id: Arc::new(Mutex::new(None)),
        }
    }

    fn add_annotation_jobs_group(
        &mut self,
        group_id: uuid::Uuid,
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    ) {
        // only add if it is not already there
        if self.groups.contains_key(&group_id) {
            return;
        }

        self.groups.insert(
            group_id,
            AnnotationJobGroup::new(group_id, feature, jobs, editor_window_uid),
        );
        if let Ok(visible_text_range) = get_visible_text_range(GetVia::Hash(editor_window_uid)) {
            self.groups
                .get_mut(&group_id)
                .unwrap() // Unwrap safe here because we just inserted the group
                .compute_annotations(&visible_text_range);
        }
    }

    fn upsert_annotation_job_group(
        &mut self,
        group_id: uuid::Uuid,
        feature: FeatureKind,
        jobs: Vec<AnnotationJob>,
        editor_window_uid: EditorWindowUid,
    ) {
        if !self.groups.contains_key(&group_id) {
            self.groups.insert(
                group_id,
                AnnotationJobGroup::new(group_id, feature, jobs, editor_window_uid),
            );
        } else {
            self.groups.get_mut(&group_id).unwrap().replace(jobs);
        }

        if let Ok(visible_text_range) = get_visible_text_range(GetVia::Hash(editor_window_uid)) {
            self.groups
                .get_mut(&group_id)
                .unwrap() // Unwrap safe here because we just inserted the group
                .compute_annotations(&visible_text_range);
        }
    }

    fn recompute_annotations(&mut self, editor_window_uid: EditorWindowUid) {
        if let Ok(visible_text_range) = get_visible_text_range(GetVia::Hash(editor_window_uid)) {
            for group in self.groups.values_mut() {
                if group.editor_window_uid() == editor_window_uid {
                    group.compute_annotations(&visible_text_range);
                }
            }
        }
    }

    fn update_annotations(&mut self, editor_window_uid: EditorWindowUid) {
        if let Ok(visible_text_range) = get_visible_text_range(GetVia::Hash(editor_window_uid)) {
            for group in self.groups.values_mut() {
                if group.editor_window_uid() == editor_window_uid {
                    group.update_annotations(&visible_text_range);
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
        get_via: GetAnnotationInGroupVia,
    ) -> Result<(), AnnotationError> {
        let annotation = self.get_annotation(group_id, get_via)?;
        {
            // If there is a another scroll to annotation job running, we want to interrupt it.
            let mut scroll_to_annotation_job_id = self.scroll_to_annotation_job_id.lock();
            if scroll_to_annotation_job_id.is_some()
                && *scroll_to_annotation_job_id != Some(annotation.id)
            {
                return Ok(());
            } else {
                *scroll_to_annotation_job_id = Some(annotation.id);
            }
        }

        tauri::async_runtime::spawn({
            let scroll_to_annotation_job_id = self.scroll_to_annotation_job_id.clone();
            async move {
                let visible_text_range = if let Ok(range) = get_visible_text_range(GetVia::Current)
                {
                    range
                } else {
                    return;
                };

                let viewport_positioning = Self::get_visibility_relative_to_viewport(
                    annotation.char_index,
                    &visible_text_range,
                );

                match viewport_positioning {
                    VisibleTextRangePositioning::Visible => {
                        _ = Self::scroll_procedure_if_annotation_within_visible_text_range(
                            &annotation,
                            group_id,
                            scroll_to_annotation_job_id,
                        )
                        .await;
                    }
                    VisibleTextRangePositioning::InvisibleAbove => {
                        _ = Self::scroll_procedure_if_annotation_outside_visible_text_range(
                            VisibleTextRangePositioning::InvisibleAbove,
                            &annotation,
                            group_id,
                            scroll_to_annotation_job_id,
                        )
                        .await;
                    }
                    VisibleTextRangePositioning::InvisibleBelow => {
                        _ = Self::scroll_procedure_if_annotation_outside_visible_text_range(
                            VisibleTextRangePositioning::InvisibleBelow,
                            &annotation,
                            group_id,
                            scroll_to_annotation_job_id,
                        )
                        .await;
                    }
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

    pub fn get_visibility_relative_to_viewport(
        char_index: usize,
        visible_text_range: &TextRange,
    ) -> VisibleTextRangePositioning {
        if char_index < visible_text_range.index {
            VisibleTextRangePositioning::InvisibleAbove
        } else if char_index > visible_text_range.index + visible_text_range.length {
            VisibleTextRangePositioning::InvisibleBelow
        } else {
            VisibleTextRangePositioning::Visible
        }
    }

    fn get_annotation(
        &mut self,
        group_id: uuid::Uuid,
        get_via: GetAnnotationInGroupVia,
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

        let job = match get_via {
            GetAnnotationInGroupVia::Id(uuid) => annotation_group.annotations.get(&uuid),
            GetAnnotationInGroupVia::Kind(kind) => annotation_group
                .annotations
                .values()
                .find(|&j| j.kind == kind),
        }
        .ok_or(AnnotationError::AnnotationNotFound)?
        .to_owned();

        Ok(job)
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
        positioning_relative_viewport: VisibleTextRangePositioning,
        annotation: &Annotation,
        group_id: uuid::Uuid,
        scroll_to_annotation_job_id: Arc<Mutex<Option<uuid::Uuid>>>,
    ) -> Result<(), AnnotationError> {
        match positioning_relative_viewport {
            VisibleTextRangePositioning::Visible => panic!("Should not happen"),
            VisibleTextRangePositioning::InvisibleAbove => {
                _ = scroll_by_one_page(true).await;
            }
            VisibleTextRangePositioning::InvisibleBelow => {
                _ = scroll_by_one_page(false).await;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(
            APPROX_SCROLL_DURATION_PAGE_UP_DOWN_MS,
        ))
        .await;

        Self::reschedule_scroll_event(scroll_to_annotation_job_id, annotation, group_id);

        Ok(())
    }

    async fn scroll_procedure_if_annotation_within_visible_text_range(
        annotation: &Annotation,
        group_id: uuid::Uuid,
        scroll_to_annotation_job_id: Arc<Mutex<Option<uuid::Uuid>>>,
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
                *scroll_to_annotation_job_id.lock() = None;
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
            APPROX_SCROLL_DURATION_PAGE_UP_DOWN_MS,
        ))
        .await;

        Self::reschedule_scroll_event(scroll_to_annotation_job_id, annotation, group_id);

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

    fn reschedule_scroll_event(
        scroll_to_annotation_job_id: Arc<Mutex<Option<uuid::Uuid>>>,
        annotation: &Annotation,
        group_id: uuid::Uuid,
    ) {
        if *scroll_to_annotation_job_id.lock() == Some(annotation.id) {
            AnnotationManagerEvent::ScrollToAnnotationInGroup((
                group_id,
                GetAnnotationInGroupVia::Id(annotation.id),
            ))
            .publish_to_tauri();
        }
    }
}
