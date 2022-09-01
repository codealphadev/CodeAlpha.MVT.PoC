use std::sync::Arc;

use anyhow::anyhow;
use parking_lot::Mutex;

use crate::{
    app_handle,
    core_engine::{
        events::{
            models::{
                DocsGeneratedMessage, RemoveCodeAnnotationMessage, UpdateCodeAnnotationMessage,
            },
            EventDocsGeneration, EventRuleExecutionState,
        },
        features::docs_generation::mintlify_documentation,
        utils::XcodeText,
        TextPosition, TextRange, WindowUid,
    },
    platform::macos::{
        get_bounds_for_TextRange, get_code_document_frame_properties, get_viewport_frame,
        get_viewport_properties, xcode::actions::replace_range_with_clipboard_text, GetVia,
        ViewportProperties,
    },
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        EventTrackingArea, TrackingArea, TrackingEventSubscription, TrackingEventType,
    },
};

use super::docs_generator::DocsGenerationError;

#[derive(Clone, Debug, PartialEq)]
pub enum DocsGenerationTaskState {
    Prepared,
    Processing,
    Finished,
}

pub struct CodeBlock {
    pub first_char_pos: TextPosition,
    pub last_char_pos: TextPosition,
    pub text: XcodeText,
}

pub struct DocsGenerationTask {
    tracking_area: TrackingArea,
    docs_insertion_point: TextRange,
    codeblock: CodeBlock,
    task_state: Arc<Mutex<DocsGenerationTaskState>>,
}

impl DocsGenerationTask {
    pub fn new(
        codeblock_first_char_position: TextPosition,
        codeblock_last_char_position: TextPosition,
        docs_insertion_point: TextRange,
        codeblock_text: XcodeText,
        text_content: &XcodeText,
        window_uid: WindowUid,
    ) -> Result<Self, DocsGenerationError> {
        let codeblock = CodeBlock {
            first_char_pos: codeblock_first_char_position,
            last_char_pos: codeblock_last_char_position,
            text: codeblock_text,
        };

        let tracking_area = Self::create_task_tracking_area(text_content, &codeblock, window_uid)?;

        Ok(Self {
            tracking_area,
            codeblock,
            task_state: Arc::new(Mutex::new(DocsGenerationTaskState::Prepared)),
            docs_insertion_point,
        })
    }

    pub fn task_state(&self) -> DocsGenerationTaskState {
        (*self.task_state.lock()).clone()
    }

    pub fn id(&self) -> uuid::Uuid {
        self.tracking_area.id
    }

    pub fn update_visualization(&self, text: &XcodeText) -> Result<(), DocsGenerationError> {
        // 1. Get the coordinates of the CodeDocumentFrame
        let code_document_frame_origin = get_code_document_frame_properties(&GetVia::Current)
            .map_err(|e| DocsGenerationError::GenericError(e.into()))?
            .dimensions
            .origin;

        // 2. Get the annotation bounds, naturally in global coordinates
        let (annotation_rect_opt, codeblock_bounds) =
            Self::calculate_annotation_bounds(text, &self.codeblock)?;

        // 3. Publish annotation_rect and codeblock_rect to frontend, this time in LOCAL coordinates. Even if empty, publish to remove ghosts from previous messages.
        EventDocsGeneration::UpdateCodeAnnotation(UpdateCodeAnnotationMessage {
            id: self.tracking_area.id,
            annotation_icon: annotation_rect_opt
                .map(|rect| rect.to_local(&code_document_frame_origin)),
            annotation_codeblock: codeblock_bounds
                .map(|rect| rect.to_local(&code_document_frame_origin)),
        })
        .publish_to_tauri(&app_handle());

        Ok(())
    }

    pub fn generate_documentation(&self) -> Result<(), DocsGenerationError> {
        let mut task_state = (self.task_state).lock();
        *task_state = DocsGenerationTaskState::Processing;

        EventRuleExecutionState::DocsGenerationStarted().publish_to_tauri(&app_handle());

        tauri::async_runtime::spawn({
            let codeblock_text_string = String::from_utf16(&self.codeblock.text)
                .map_err(|err| DocsGenerationError::GenericError(err.into()))?;
            let docs_insertion_point = self.docs_insertion_point;
            let task_state = self.task_state.clone();
            let task_id = self.id();
            async move {
                let mut mintlify_response =
                    mintlify_documentation(&codeblock_text_string, None).await;

                if let Ok(mintlify_response) = &mut mintlify_response {
                    // Paste it at the docs insertion point
                    replace_range_with_clipboard_text(
                        &app_handle(),
                        &GetVia::Current,
                        &docs_insertion_point,
                        Some(&mintlify_response.docstring),
                        true,
                    )
                    .await;

                    EventDocsGeneration::DocsGenerated(DocsGeneratedMessage {
                        id: task_id,
                        text: mintlify_response.preview.to_owned(),
                    })
                    .publish_to_tauri(&app_handle());

                    // Notifiy the frontend that the task is finished
                    EventRuleExecutionState::DocsGenerationFinished()
                        .publish_to_tauri(&app_handle());
                }
                (*task_state.lock()) = DocsGenerationTaskState::Finished;
            }
        });

        Ok(())
    }

    pub fn create_task_tracking_area(
        text: &XcodeText,
        code_block: &CodeBlock,
        window_uid: WindowUid,
    ) -> Result<TrackingArea, DocsGenerationError> {
        // When we create the annotation, we need to compute the bounds for the frontend so it knows where to display the annotation
        // and we need to create a tracking area which the tracking area manager takes care of. The tracking area manager uses GLOBAL coordinates
        // whereas the frontend uses LOCAL coordinates; local to the CodeDocumentFrame.

        // 1. Get the annotation bounds, naturally in global coordinates
        let (annotation_rect_opt, _) = Self::calculate_annotation_bounds(text, code_block)?;

        let tracking_area = TrackingArea {
            id: uuid::Uuid::new_v4(),
            window_uid,
            rectangles: annotation_rect_opt.map_or(vec![], |rect| vec![rect]),
            event_subscriptions: TrackingEventSubscription::TrackingEventTypes(vec![
                TrackingEventType::MouseClicked,
                TrackingEventType::MouseEntered,
                TrackingEventType::MouseExited,
            ]),
        };

        // 3. Publish to the tracking area manager with its original GLOBAL coordinates
        EventTrackingArea::Add(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());

        Ok(tracking_area)
    }

    pub fn update_task_tracking_area(
        &mut self,
        text: &XcodeText,
    ) -> Result<(), DocsGenerationError> {
        // 2. Get the local coordinates of the AnnotationSectionFrame
        if let Ok((annotation_rect_opt, _)) =
            Self::calculate_annotation_bounds(text, &self.codeblock)
        {
            self.tracking_area.rectangles = annotation_rect_opt.map_or(vec![], |rect| vec![rect]);

            // Check if the annotation is outside of the viewport and if so, remove the tracking areas
            let viewport = get_viewport_frame(&GetVia::Current).map_err(|_| {
                DocsGenerationError::GenericError(anyhow!("Could not derive textarea dimensions"))
            })?;

            if let Some(annotation_rect) = annotation_rect_opt {
                if !viewport.contains_position(&annotation_rect.top_left())
                    && !viewport.contains_position(&annotation_rect.bottom_left())
                {
                    self.tracking_area.rectangles = vec![];
                }
            }

            EventTrackingArea::Update(vec![self.tracking_area.clone()])
                .publish_to_tauri(&app_handle());

            Ok(())
        } else {
            Err(DocsGenerationError::DocsGenTaskUpdateFailed)
        }
    }

    /// It calculates the bounds of the annotation icon and the codeblock rectangle
    /// The annotation icon is going to be the TrackingArea's rectangle. The codeblock rectangle is
    /// the one that is going to be highlighted.
    fn calculate_annotation_bounds(
        text: &XcodeText,
        code_block: &CodeBlock,
    ) -> Result<(Option<LogicalFrame>, Option<LogicalFrame>), DocsGenerationError> {
        // 1. Get viewport dimensions
        let ViewportProperties {
            dimensions: viewport,
            annotation_section,
            code_section: _,
        } = get_viewport_properties(&GetVia::Current).map_err(|_| {
            DocsGenerationError::GenericError(anyhow!("Could not derive textarea dimensions"))
        })?;

        // 2. Calculate the annotation rectangles
        if let (Some(first_char_text_pos), Some(last_char_text_pos)) = (
            code_block.first_char_pos.as_TextIndex(&text),
            code_block.last_char_pos.as_TextIndex(&text),
        ) {
            let first_char_bounds_opt = get_bounds_for_TextRange(
                &TextRange {
                    index: first_char_text_pos,
                    length: 1,
                },
                &GetVia::Current,
            );

            let last_char_bounds_opt = get_bounds_for_TextRange(
                &TextRange {
                    index: last_char_text_pos,
                    length: 1,
                },
                &GetVia::Current,
            );
            let codeblock_top = if let Ok(first_char_bounds) = first_char_bounds_opt {
                f64::max(
                    viewport.origin.y,
                    first_char_bounds.origin.y + first_char_bounds.size.height,
                )
            } else {
                viewport.origin.y
            };
            let codeblock_bottom = if let Ok(last_char_bounds) = last_char_bounds_opt {
                f64::min(
                    last_char_bounds.origin.y + last_char_bounds.size.height,
                    viewport.origin.y + viewport.size.height,
                )
            } else {
                viewport.origin.y + viewport.size.height
            };

            let codeblock_bounds = Some(LogicalFrame {
                origin: LogicalPosition {
                    x: viewport.origin.x,
                    y: codeblock_top,
                },
                size: LogicalSize {
                    width: annotation_section
                        .expect("Non fast track should return annotation_section")
                        .size
                        .width,
                    height: codeblock_bottom - codeblock_top,
                },
            });

            let annotation_bounds = if let Ok(first_char_bounds) = first_char_bounds_opt {
                Some(LogicalFrame {
                    origin: LogicalPosition {
                        x: viewport.origin.x,
                        y: first_char_bounds.origin.y,
                    },
                    size: LogicalSize {
                        width: annotation_section
                            .expect("Non fast track should return annotation_section")
                            .size
                            .width,
                        height: first_char_bounds.size.height,
                    },
                })
            } else {
                None
            };

            return Ok((annotation_bounds, codeblock_bounds));
        } else {
            return Err(DocsGenerationError::GenericError(anyhow!(
                "Could not get text range of the codeblock"
            )));
        }
    }
}

impl Drop for DocsGenerationTask {
    fn drop(&mut self) {
        EventDocsGeneration::RemoveCodeAnnotation(RemoveCodeAnnotationMessage { id: self.id() })
            .publish_to_tauri(&app_handle());
        EventTrackingArea::Remove(vec![self.id()]).publish_to_tauri(&app_handle());
    }
}
