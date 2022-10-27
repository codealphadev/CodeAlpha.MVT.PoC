use anyhow::anyhow;
use log::error;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use ts_rs::TS;

use crate::{
    core_engine::{features::FeatureKind, EditorWindowUid, TextRange},
    platform::macos::{
        get_bounds_for_TextRange, get_focused_window, get_minimal_viewport_properties,
        get_number_of_characters, get_visible_text_range, scroll_by_one_page, GetVia, XcodeError,
    },
    utils::geometry::{LogicalFrame, LogicalPosition},
};

use super::{
    listeners::{annotation_events::annotation_events_listener, xcode::xcode_listener},
    AnnotationJob, AnnotationJobGroup, AnnotationJobGroupTrait, AnnotationKind,
    VisibleTextRangePositioning,
};

#[derive(thiserror::Error, Debug)]
pub enum AnnotationError {
    #[error("Annotation of the given job uid does not exist.")]
    AnnotationNotFound,
    #[error("AnnotationGroup of the given group uid does not exist.")]
    AnnotationGroupNotFound,
    #[error("The annotation char index is larger than the number of characters in the textarea.")]
    AnnotationOutOfReach,
    #[error("AnnotationGroup of the given uid does not exist.")]
    AnnotationJobGroupNotFound,
    #[error("Annotation not related to currently focused editor windw.")]
    AnnotationOnDifferentEditorWindow,
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
#[derive(Clone, Debug, PartialEq)]
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
        cancel_recv: oneshot::Receiver<&'static ()>,
    ) -> Result<(), AnnotationError>;
}

pub struct AnnotationsManager {
    groups: HashMap<uuid::Uuid, AnnotationJobGroup>,

    cancel_scrolling_task_sender: Option<oneshot::Sender<&'static ()>>,
}

impl AnnotationsManagerTrait for AnnotationsManager {
    fn new() -> Self {
        Self {
            groups: HashMap::new(),
            cancel_scrolling_task_sender: None,
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
        cancel_recv: oneshot::Receiver<&'static ()>,
    ) -> Result<(), AnnotationError> {
        let annotation = self.get_annotation(group_id, get_via)?;
        let annotation_group = self.get_annotation_group(group_id)?;

        let (perform_scrolling_send, mut perform_scrolling_recv) = mpsc::channel(1);

        tauri::async_runtime::spawn({
            async move {
                tauri::async_runtime::spawn({
                    async move {
                        loop {
                            if Self::scroll_check(&annotation, &annotation_group).is_err() {
                                break;
                            }

                            let viewport_positioning = if let Ok(positioning) =
                                Self::get_visibility_relative_to_viewport(
                                    annotation.char_index,
                                    None,
                                ) {
                                positioning
                            } else {
                                break;
                            };

                            let positioning = match viewport_positioning {
                                VisibleTextRangePositioning::Visible => {
                                    Self::scroll_procedure_if_annotation_within_visible_text_range(
                                        &annotation,
                                        perform_scrolling_send.clone(),
                                    )
                                    .await
                                }
                                VisibleTextRangePositioning::InvisibleAbove => {
                                    Self::scroll_procedure_if_annotation_outside_visible_text_range(
                                        VisibleTextRangePositioning::InvisibleAbove,
                                        perform_scrolling_send.clone(),
                                    )
                                    .await
                                }
                                VisibleTextRangePositioning::InvisibleBelow => {
                                    Self::scroll_procedure_if_annotation_outside_visible_text_range(
                                        VisibleTextRangePositioning::InvisibleBelow,
                                        perform_scrolling_send.clone(),
                                    )
                                    .await
                                }
                            };

                            if let Ok(positioning) = positioning {
                                if positioning == ViewportPositioning::Visible {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                });

                tokio::select! {
                    _ = perform_scrolling_recv.recv() => {
                        // Scrolling finished
                    }
                    _ = cancel_recv => {
                        // Scrolling cancelled
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

    pub fn reset_scroll_cancel_channel(&mut self) -> oneshot::Receiver<&'static ()> {
        if let Some(sender) = self.cancel_scrolling_task_sender.take() {
            // Cancel previous task if it exists.
            _ = sender.send(&());
        }

        let (send, recv) = oneshot::channel();
        self.cancel_scrolling_task_sender = Some(send);
        recv
    }

    pub fn get_visibility_relative_to_viewport(
        char_index: usize,
        visible_text_range: Option<&TextRange>,
    ) -> Result<VisibleTextRangePositioning, AnnotationError> {
        let visible_text_range = if let Some(visible_text_range) = visible_text_range {
            visible_text_range.to_owned()
        } else {
            get_visible_text_range(GetVia::Current)
                .map_err(|e| AnnotationError::GenericError(e.into()))?
        };

        if char_index < visible_text_range.index {
            Ok(VisibleTextRangePositioning::InvisibleAbove)
        } else if char_index > visible_text_range.index + visible_text_range.length {
            Ok(VisibleTextRangePositioning::InvisibleBelow)
        } else {
            Ok(VisibleTextRangePositioning::Visible)
        }
    }

    fn get_annotation_group(
        &self,
        group_id: uuid::Uuid,
    ) -> Result<AnnotationGroup, AnnotationError> {
        let job_group = self
            .groups
            .get(&group_id)
            .ok_or(AnnotationError::AnnotationJobGroupNotFound)?;

        job_group
            .get_annotation_group()
            .ok_or(AnnotationError::AnnotationGroupNotFound)
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

    fn scroll_check(
        annotation: &Annotation,
        annotation_group: &AnnotationGroup,
    ) -> Result<(), AnnotationError> {
        // Check if annotation group is tied to currently focused editor window
        if annotation_group.editor_window_uid
            != get_focused_window().map_err(|e| {
                AnnotationError::GenericError(anyhow!("Could not get focused window: {}", e))
            })?
        {
            return Err(AnnotationError::AnnotationOnDifferentEditorWindow);
        }

        // Safety net: check if annotation char_index is even reachable
        if annotation.char_index as i32
            >= get_number_of_characters(GetVia::Current)
                .map_err(|e| AnnotationError::GenericError(e.into()))?
        {
            return Err(AnnotationError::AnnotationOutOfReach);
        }

        Ok(())
    }

    async fn scroll_procedure_if_annotation_outside_visible_text_range(
        positioning_relative_viewport: VisibleTextRangePositioning,
        sender: mpsc::Sender<()>,
    ) -> Result<ViewportPositioning, AnnotationError> {
        let viewport_positioning;
        match positioning_relative_viewport {
            VisibleTextRangePositioning::Visible => {
                panic!("Should not happen");
            }
            VisibleTextRangePositioning::InvisibleAbove => {
                scroll_by_one_page(true, sender).await?;
                viewport_positioning = ViewportPositioning::InvisibleAbove;
            }
            VisibleTextRangePositioning::InvisibleBelow => {
                scroll_by_one_page(false, sender).await?;
                viewport_positioning = ViewportPositioning::InvisibleBelow;
            }
        }

        Ok(viewport_positioning)
    }

    async fn scroll_procedure_if_annotation_within_visible_text_range(
        annotation: &Annotation,
        sender: mpsc::Sender<()>,
    ) -> Result<ViewportPositioning, AnnotationError> {
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

        let viewport_position = Self::annotation_visible_on_viewport(annotation_top_left)?;

        match viewport_position {
            ViewportPositioning::Visible => {
                // Do nothing
            }
            ViewportPositioning::InvisibleAbove => {
                scroll_by_one_page(true, sender).await?;
            }
            ViewportPositioning::InvisibleBelow => {
                scroll_by_one_page(false, sender).await?;
            }
        }

        Ok(viewport_position)
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
