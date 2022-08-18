use std::sync::{Arc, Mutex};

use crate::{
    app_handle,
    ax_interaction::{
        derive_xcode_textarea_dimensions, get_textarea_uielement,
        xcode::actions::replace_range_with_clipboard_text,
    },
    core_engine::{
        ax_utils::get_bounds_of_TextRange,
        events::{
            models::{CodeAnnotationMessage, DocsGeneratedMessage, RemoveCodeAnnotationMessage},
            EventDocsGeneration, EventRuleExecutionState,
        },
        features::docs_generation::mintlify_documentation,
        rules::{TextPosition, TextRange},
        types::MatchRectangle,
        utils::XcodeText,
    },
    utils::geometry::{LogicalPosition, LogicalSize},
    window_controls::{
        EventTrackingArea, TrackingArea, TrackingEventSubscription, TrackingEventType,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub enum DocsGenerationTaskState {
    Uninitialized,
    Prepared,
    Processing,
    Finished,
    Canceled,
}

pub struct DocsGenerationTask {
    // The tracking area attached to the task. Is set to none, if it can not
    tracking_area: Option<TrackingArea>, // TODO: Don't think this should be optional. Guard in the factory pattern constructor.
    docs_insertion_point: TextRange,
    _docs_indentation: usize,
    codeblock_first_char_pos: TextPosition,
    codeblock_last_char_pos: TextPosition,
    codeblock_text: XcodeText,
    pid: i32,
    task_state: Arc<Mutex<DocsGenerationTaskState>>,
}

type AnnotationIconRectangle = MatchRectangle;
type CodeblockRectangle = MatchRectangle;

impl DocsGenerationTask {
    pub fn new(
        pid: i32,
        codeblock_first_char_position: TextPosition,
        codeblock_last_char_position: TextPosition,
        docs_insertion_point: TextRange,
        _docs_indentation: usize,
        codeblock_text: XcodeText,
    ) -> Self {
        Self {
            tracking_area: None,
            codeblock_first_char_pos: codeblock_first_char_position,
            codeblock_last_char_pos: codeblock_last_char_position,
            codeblock_text,
            pid,
            task_state: Arc::new(Mutex::new(DocsGenerationTaskState::Uninitialized)),
            docs_insertion_point,
            _docs_indentation,
        }
    }

    pub fn task_state(&self) -> DocsGenerationTaskState {
        (*self.task_state.lock().unwrap()).clone()
    }

    pub fn id(&self) -> Option<uuid::Uuid> {
        if let Some(tracking_area) = self.tracking_area.as_ref() {
            Some(tracking_area.id)
        } else {
            None
        }
    }

    fn is_frozen(&self) -> bool {
        let task_state = (self.task_state).lock().unwrap();

        *task_state == DocsGenerationTaskState::Finished
            || *task_state == DocsGenerationTaskState::Canceled
            || *task_state == DocsGenerationTaskState::Processing
    }

    pub fn generate_documentation(&mut self) {
        let mut task_state = (self.task_state).lock().unwrap();
        *task_state = DocsGenerationTaskState::Processing;

        EventRuleExecutionState::DocsGenerationStarted().publish_to_tauri(&app_handle());

        let task_id = if let Some(id) = self.id() {
            id
        } else {
            return;
        };

        let codeblock_text_string =
            String::from_utf16(&self.codeblock_text).expect("`codeblock_text` is not valid UTF-16");
        let pid_move_copy = self.pid;
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
                    pid_move_copy,
                    &docs_insertion_point_move_copy,
                    Some(&mintlify_response.docstring),
                    true,
                );

                // Publish annotation_rect and codeblock_rect to frontend
                EventDocsGeneration::DocsGenerated(DocsGeneratedMessage {
                    id: task_id,
                    text: mintlify_response.preview.to_owned(),
                })
                .publish_to_tauri(&app_handle());

                // Notifiy the frontend that the file has been formatted successfully
                EventRuleExecutionState::DocsGenerationFinished().publish_to_tauri(&app_handle());
            }
            (*task_state.lock().unwrap()) = DocsGenerationTaskState::Finished;
        });
    }

    pub fn create_code_annotation(&mut self, text: &XcodeText) -> Result<(), &str> {
        if self.tracking_area.is_some() || self.is_frozen() {
            return Err("Task is frozen or tracking");
        }
        let (annotation_rect_opt, codeblock_rect_opt) = self.calculate_annotation_bounds(text)?;

        let tracking_area = if annotation_rect_opt.is_some() && codeblock_rect_opt.is_some() {
            TrackingArea {
                id: uuid::Uuid::new_v4(),
                rectangles: vec![annotation_rect_opt.expect("Previously checked for Some")],
                event_subscriptions: TrackingEventSubscription::TrackingEventTypes(vec![
                    TrackingEventType::MouseClicked,
                    TrackingEventType::MouseEntered,
                    TrackingEventType::MouseExited,
                ]),
            }
        } else {
            // The codeblock has been scrolled out of view. Still create TrackingArea but don't subscribe to Tracking Events.
            TrackingArea {
                id: uuid::Uuid::new_v4(),
                rectangles: vec![],
                event_subscriptions: TrackingEventSubscription::None,
            }
        };

        EventTrackingArea::Add(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());

        // Publish annotation_rect and codeblock_rect to frontend. Even if empty, publish to remove ghosts from previous messages.
        EventDocsGeneration::UpdateCodeAnnotation(CodeAnnotationMessage {
            id: tracking_area.id,
            annotation_icon: annotation_rect_opt,
            annotation_codeblock: codeblock_rect_opt,
        })
        .publish_to_tauri(&app_handle());

        self.tracking_area = Some(tracking_area);
        let mut task_state = (self.task_state).lock().unwrap();
        *task_state = DocsGenerationTaskState::Prepared;

        Ok(())
    }

    pub fn update_code_annotation_position(&mut self, text: &XcodeText) -> bool {
        if self.tracking_area.is_none() {
            return false;
        };

        if let Ok((annotation_rect_opt, codeblock_rect_opt)) =
            self.calculate_annotation_bounds(text)
        {
            let tracking_area = if let Some(tracking_area) = &mut self.tracking_area {
                tracking_area
            } else {
                return false;
            };

            // If we receive NONE for the annotation and codeblock rectangles, the reason is that the codeblock has been scrolled out of view.
            // In this case, we still need to keep and update the TrackingArea but unsubscribe from the TrackingEvents for now.
            if let (Some(annotation_rect), Some(codeblock_rect)) =
                (annotation_rect_opt, codeblock_rect_opt)
            {
                // Update the tracking area
                tracking_area.rectangles = vec![annotation_rect];

                EventTrackingArea::Update(vec![tracking_area.clone()])
                    .publish_to_tauri(&app_handle());

                // Publish annotation_rect and codeblock_rect to frontend
                EventDocsGeneration::UpdateCodeAnnotation(CodeAnnotationMessage {
                    id: tracking_area.id,
                    annotation_icon: Some(annotation_rect),
                    annotation_codeblock: Some(codeblock_rect),
                })
                .publish_to_tauri(&app_handle());

                true
            } else {
                // Update the tracking area
                tracking_area.rectangles = vec![];
                tracking_area.event_subscriptions = TrackingEventSubscription::None;

                EventTrackingArea::Update(vec![tracking_area.clone()])
                    .publish_to_tauri(&app_handle());

                // Hide the annotation_icon and annotation_codeblock from the frontend
                EventDocsGeneration::UpdateCodeAnnotation(CodeAnnotationMessage {
                    id: tracking_area.id,
                    annotation_icon: None,
                    annotation_codeblock: None,
                })
                .publish_to_tauri(&app_handle());

                true
            }
        } else {
            // Remove the tracking area
            EventTrackingArea::Remove(vec![self.tracking_area.as_ref().unwrap().id])
                .publish_to_tauri(&app_handle());
            self.tracking_area = None;
            let mut task_state = (self.task_state).lock().unwrap();
            *task_state = DocsGenerationTaskState::Canceled;

            // Remove the annotation from the frontend
            EventDocsGeneration::UpdateCodeAnnotation(CodeAnnotationMessage {
                id: self.tracking_area.as_ref().unwrap().id,
                annotation_icon: None,
                annotation_codeblock: None,
            })
            .publish_to_tauri(&app_handle());

            false
        }
    }

    /// It calculates the bounds of the annotation icon and the codeblock rectangle
    /// The annotation icon is going to be the TrackingArea's rectangle. The codeblock rectangle is
    /// the one that is going to be highlighted.
    fn calculate_annotation_bounds(
        &self,
        text: &XcodeText,
    ) -> Result<(Option<AnnotationIconRectangle>, Option<CodeblockRectangle>), &'static str> {
        // 1. Get textarea dimensions
        let textarea_ui_element = if let Some(elem) = get_textarea_uielement(self.pid) {
            elem
        } else {
            return Ok((None, None));
        };

        let (textarea_origin, textarea_size) =
            if let Ok((origin, size)) = derive_xcode_textarea_dimensions(&textarea_ui_element) {
                (origin, size)
            } else {
                return Ok((None, None));
            };

        // 2. Calculate the annotation rectangles
        if let (Some(first_char_text_pos), Some(last_char_text_pos)) = (
            self.codeblock_first_char_pos.as_TextIndex(&text),
            self.codeblock_last_char_pos.as_TextIndex(&text),
        ) {
            let first_char_bounds = get_bounds_of_TextRange(
                &TextRange {
                    index: first_char_text_pos,
                    length: 1,
                },
                &textarea_ui_element,
            );

            let last_char_bounds = get_bounds_of_TextRange(
                &TextRange {
                    index: last_char_text_pos,
                    length: 1,
                },
                &textarea_ui_element,
            );

            if let (Some(first_char_bounds), Some(last_char_bounds)) =
                (first_char_bounds, last_char_bounds)
            {
                // 2.1 Annotation rectangle of the codeblock
                // Height: the height of the codeblock minus the first line height
                // Width: the width of the first character
                // Position: left of the codeblock
                let codeblock_bounds = CodeblockRectangle {
                    origin: LogicalPosition {
                        x: textarea_origin.x,
                        y: first_char_bounds.origin.y + first_char_bounds.size.height,
                    },
                    size: LogicalSize {
                        width: first_char_bounds.size.width,
                        height: last_char_bounds.origin.y - first_char_bounds.origin.y
                            + first_char_bounds.size.height,
                    },
                };

                // 2.2 Annotation rectangle of the docs generation icon
                let annotation_bounds = AnnotationIconRectangle {
                    origin: LogicalPosition {
                        x: textarea_origin.x,
                        y: first_char_bounds.origin.y,
                    },
                    size: LogicalSize {
                        width: first_char_bounds.size.height / 1.5, // This factor brings it 12px width on 100% zoom level.
                        height: first_char_bounds.size.height,
                    },
                };

                // 2.3 Check if the annotation_bounds are valid; bounds are within the visible textarea
                if annotation_bounds.origin.y < textarea_origin.y
                    || annotation_bounds.origin.y > textarea_origin.y + textarea_size.height
                {
                    return Ok((None, None));
                }

                return Ok((Some(annotation_bounds), Some(codeblock_bounds)));
            } else {
                return Ok((None, None));
            }
        } else {
            return Err("Could not get text range of the codeblock");
        }
    }
}

impl Drop for DocsGenerationTask {
    fn drop(&mut self) {
        println!("DocsGenerationTask::drop");
        if let Some(tracking_area) = self.tracking_area.as_ref() {
            EventDocsGeneration::RemoveCodeAnnotation(RemoveCodeAnnotationMessage {
                id: tracking_area.id,
            })
            .publish_to_tauri(&app_handle());
            EventTrackingArea::Remove(vec![tracking_area.id]).publish_to_tauri(&app_handle());
        }
    }
}
