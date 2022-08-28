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
        get_bounds_for_TextRange, get_viewport_frame,
        xcode::actions::replace_range_with_clipboard_text, GetVia,
    },
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        EventTrackingArea, TrackingArea, TrackingEventSubscription, TrackingEventType,
    },
};

use super::docs_generator::DocsGenerationError;

#[derive(Clone, Debug, PartialEq)]
pub enum DocsGenerationTaskState {
    Uninitialized,
    Prepared,
    Processing,
    Finished,
    Canceled,
}

pub struct DocsGenerationTask {
    tracking_area: Option<TrackingArea>, // TODO: Don't think this should be optional. Guard in the factory pattern constructor.
    docs_insertion_point: TextRange,
    codeblock_first_char_pos: TextPosition,
    codeblock_last_char_pos: TextPosition,
    codeblock_text: XcodeText,
    task_state: Arc<Mutex<DocsGenerationTaskState>>,
}

impl DocsGenerationTask {
    pub fn new(
        codeblock_first_char_position: TextPosition,
        codeblock_last_char_position: TextPosition,
        docs_insertion_point: TextRange,
        codeblock_text: XcodeText,
    ) -> Self {
        Self {
            tracking_area: None,
            codeblock_first_char_pos: codeblock_first_char_position,
            codeblock_last_char_pos: codeblock_last_char_position,
            codeblock_text,
            task_state: Arc::new(Mutex::new(DocsGenerationTaskState::Uninitialized)),
            docs_insertion_point,
        }
    }

    pub fn task_state(&self) -> DocsGenerationTaskState {
        (*self.task_state.lock()).clone()
    }

    pub fn id(&self) -> Option<uuid::Uuid> {
        if let Some(tracking_area) = self.tracking_area.as_ref() {
            Some(tracking_area.id)
        } else {
            None
        }
    }

    fn is_frozen(&self) -> bool {
        let task_state = (self.task_state).lock();

        *task_state == DocsGenerationTaskState::Finished
            || *task_state == DocsGenerationTaskState::Canceled
            || *task_state == DocsGenerationTaskState::Processing
    }

    pub fn generate_documentation(&mut self) {
        let mut task_state = (self.task_state).lock();
        *task_state = DocsGenerationTaskState::Processing;

        EventRuleExecutionState::DocsGenerationStarted().publish_to_tauri(&app_handle());

        let task_id = if let Some(id) = self.id() {
            id
        } else {
            return;
        };

        let codeblock_text_string =
            String::from_utf16(&self.codeblock_text).expect("`codeblock_text` is not valid UTF-16");
        let docs_insertion_point_move_copy = self.docs_insertion_point.clone();
        let task_state = self.task_state.clone();
        tauri::async_runtime::spawn(async move {
            let mut mintlify_response = mintlify_documentation(&codeblock_text_string, None).await;

            if let Ok(mintlify_response) = &mut mintlify_response {
                // add newline character at the end of mintlify_response.docstring
                // mintlify_response.docstring.push('\n');
                // mintlify_response.docstring.push('\t');

                // add spaces at the end of the docstring equal to the column of the codeblock_first_char to have a correct indentation after the paste operation
                // for _ in 0..self.docs_indentation {
                //     mintlify_response.docstring.push(' ');
                // }

                // Paste it at the docs insertion point
                replace_range_with_clipboard_text(
                    &app_handle(),
                    &GetVia::Current,
                    &docs_insertion_point_move_copy,
                    Some(&mintlify_response.docstring),
                    true,
                )
                .await;

                // Publish annotation_rect and codeblock_rect to frontend
                EventDocsGeneration::DocsGenerated(DocsGeneratedMessage {
                    id: task_id,
                    text: mintlify_response.preview.to_owned(),
                })
                .publish_to_tauri(&app_handle());

                // Notifiy the frontend that the file has been formatted successfully
                EventRuleExecutionState::DocsGenerationFinished().publish_to_tauri(&app_handle());
            }
            (*task_state.lock()) = DocsGenerationTaskState::Finished;
        });
    }

    pub fn create_code_annotation(
        &mut self,
        text: &XcodeText,
        window_uid: WindowUid,
    ) -> Result<(), &str> {
        if self.tracking_area.is_some() || self.is_frozen() {
            return Err("Task is frozen or tracking");
        }
        let (annotation_rect_opt, codeblock_bounds) = self.calculate_annotation_bounds(text)?;
        let rectangles = if let Some(annotation_rect) = annotation_rect_opt {
            vec![annotation_rect]
        } else {
            vec![]
        };
        let tracking_area = TrackingArea {
            id: uuid::Uuid::new_v4(),
            window_uid: window_uid,
            rectangles,
            event_subscriptions: TrackingEventSubscription::TrackingEventTypes(vec![
                TrackingEventType::MouseClicked,
                TrackingEventType::MouseEntered,
                TrackingEventType::MouseExited,
            ]),
        };

        EventTrackingArea::Add(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());

        // Publish annotation_rect and codeblock_rect to frontend. Even if empty, publish to remove ghosts from previous messages.
        EventDocsGeneration::UpdateCodeAnnotation(UpdateCodeAnnotationMessage {
            id: tracking_area.id,
            annotation_icon: annotation_rect_opt,
            annotation_codeblock: codeblock_bounds,
        })
        .publish_to_tauri(&app_handle());

        self.tracking_area = Some(tracking_area);
        let mut task_state = (self.task_state).lock();
        *task_state = DocsGenerationTaskState::Prepared;

        Ok(())
    }

    pub fn update_code_annotation_position(
        &mut self,
        text: &XcodeText,
    ) -> Result<(), DocsGenerationError> {
        let mut tracking_area_copy =
            self.tracking_area
                .clone()
                .ok_or(DocsGenerationError::GenericError(anyhow!(
                    "No tracking area"
                )))?;

        if let Ok((annotation_rect_opt, codeblock_rect_opt)) =
            self.calculate_annotation_bounds(text)
        {
            tracking_area_copy.rectangles = if let Some(annotation_rect) = annotation_rect_opt {
                vec![annotation_rect]
            } else {
                vec![]
            };

            EventTrackingArea::Update(vec![tracking_area_copy.clone()])
                .publish_to_tauri(&app_handle());

            // Publish annotation_rect and codeblock_rect to frontend
            EventDocsGeneration::UpdateCodeAnnotation(UpdateCodeAnnotationMessage {
                id: tracking_area_copy.id,
                annotation_icon: annotation_rect_opt,
                annotation_codeblock: codeblock_rect_opt,
            })
            .publish_to_tauri(&app_handle());
        } else {
            // Remove the tracking area
            EventTrackingArea::Remove(vec![tracking_area_copy.id]).publish_to_tauri(&app_handle());
            let mut task_state = (self.task_state).lock();
            *task_state = DocsGenerationTaskState::Canceled;

            // Remove the annotation from the frontend
            EventDocsGeneration::RemoveCodeAnnotation(RemoveCodeAnnotationMessage {
                id: tracking_area_copy.id,
            })
            .publish_to_tauri(&app_handle());

            self.tracking_area = None;
        }
        Ok(())
    }

    /// It calculates the bounds of the annotation icon and the codeblock rectangle
    /// The annotation icon is going to be the TrackingArea's rectangle. The codeblock rectangle is
    /// the one that is going to be highlighted.
    fn calculate_annotation_bounds(
        &self,
        text: &XcodeText,
    ) -> Result<(Option<LogicalFrame>, Option<LogicalFrame>), &'static str> {
        let (textarea_origin, textarea_size) =
            if let Ok(code_section_frame) = get_viewport_frame(&GetVia::Current) {
                (code_section_frame.origin, code_section_frame.size)
            } else {
                return Err("Could not derive textarea dimensions");
            };

        // 2. Calculate the annotation rectangles
        if let (Some(first_char_text_pos), Some(last_char_text_pos)) = (
            self.codeblock_first_char_pos.as_TextIndex(&text),
            self.codeblock_last_char_pos.as_TextIndex(&text),
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
                    textarea_origin.y,
                    first_char_bounds.origin.y + first_char_bounds.size.height,
                )
            } else {
                textarea_origin.y
            };
            let codeblock_bottom = if let Ok(last_char_bounds) = last_char_bounds_opt {
                f64::min(
                    last_char_bounds.origin.y + last_char_bounds.size.height,
                    textarea_origin.y + textarea_size.height,
                )
            } else {
                textarea_origin.y + textarea_size.height
            };

            let char_width = if let Ok(first_char_bounds) = first_char_bounds_opt {
                first_char_bounds.size.height / 1.5
            } else if let Ok(last_char_bounds) = last_char_bounds_opt {
                last_char_bounds.size.height / 1.5
            } else {
                12.0 // Fallback - should be rare
            };

            let codeblock_bounds = Some(LogicalFrame {
                origin: LogicalPosition {
                    x: textarea_origin.x,
                    y: codeblock_top,
                },
                size: LogicalSize {
                    width: char_width,
                    height: codeblock_bottom - codeblock_top,
                },
            });

            let annotation_bounds = if let Ok(first_char_bounds) = first_char_bounds_opt {
                Some(LogicalFrame {
                    origin: LogicalPosition {
                        x: textarea_origin.x,
                        y: first_char_bounds.origin.y,
                    },
                    size: LogicalSize {
                        width: char_width, // This factor brings it 12px width on 100% zoom level.
                        height: first_char_bounds.size.height,
                    },
                })
            } else {
                None
            };

            return Ok((annotation_bounds, codeblock_bounds));
        } else {
            return Err("Could not get text range of the codeblock");
        }
    }
}

impl Drop for DocsGenerationTask {
    fn drop(&mut self) {
        if let Some(tracking_area) = self.tracking_area.as_ref() {
            EventDocsGeneration::RemoveCodeAnnotation(RemoveCodeAnnotationMessage {
                id: tracking_area.id,
            })
            .publish_to_tauri(&app_handle());
            EventTrackingArea::Remove(vec![tracking_area.id]).publish_to_tauri(&app_handle());
        }
    }
}
