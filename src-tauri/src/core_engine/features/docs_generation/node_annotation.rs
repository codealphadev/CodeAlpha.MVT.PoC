use std::sync::Arc;

use anyhow::anyhow;
use parking_lot::Mutex;
use tracing::debug;

use crate::{
    app_handle,
    core_engine::{
        events::{
            models::{
                NodeExplanationFetchedMessage, RemoveNodeAnnotationMessage,
                UpdateNodeAnnotationMessage, UpdateNodeExplanationMessage,
            },
            EventRuleExecutionState, NodeAnnotationEvent, NodeExplanationEvent,
        },
        syntax_tree::{FunctionParameter, SwiftCodeBlockKind},
        utils::XcodeText,
        EditorWindowUid, TextPosition, TextRange,
    },
    platform::macos::{
        get_bounds_for_TextRange, get_code_document_frame_properties, get_viewport_frame,
        get_viewport_properties, GetVia, ViewportProperties,
    },
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::AppWindow, EventTrackingArea, TrackingArea, TrackingEventSubscription,
        TrackingEventType,
    },
    NODE_EXPLANATION_CURRENT_INSERTION_POINT,
};

use super::{
    docs_generator::{compute_docs_insertion_point_and_indentation, DocsGenerationError},
    fetch_node_explanation, NodeExplanation,
};

#[derive(Clone, Debug, PartialEq)]
pub enum NodeAnnotationState {
    New,
    FetchingExplanation,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnnotationCodeBlock {
    pub name: Option<String>,
    pub func_parameters_todo: Option<Vec<FunctionParameter>>, // TODO: COD-320 Majorly refactor CodeBlock. Not ok to allow incompatible kind and parameters etc.
    pub func_complexity_todo: Option<isize>, // TODO: COD-320 Majorly refactor CodeBlock. Not ok to allow incompatible kind and parameters etc.
    pub first_char_pos: TextPosition,
    pub last_char_pos: TextPosition,
    pub kind: SwiftCodeBlockKind,
    pub text: XcodeText,
    pub context: Option<XcodeText>,
}

#[derive(Debug, Clone)]
pub struct NodeAnnotation {
    tracking_area: TrackingArea,
    node_code_block: AnnotationCodeBlock,
    state: Arc<Mutex<NodeAnnotationState>>,
    explanation: Arc<Mutex<Option<NodeExplanation>>>,
}

impl PartialEq for NodeAnnotation {
    fn eq(&self, other: &Self) -> bool {
        self.tracking_area.eq_props(&other.tracking_area)
            && self.node_code_block == other.node_code_block
    }
}

impl NodeAnnotation {
    pub fn new(
        codeblock: AnnotationCodeBlock,
        text_content: &XcodeText,
        window_uid: EditorWindowUid,
    ) -> Result<Self, DocsGenerationError> {
        let tracking_area = Self::create_tracking_area(text_content, &codeblock, window_uid)?;

        Ok(Self {
            tracking_area,
            node_code_block: codeblock,
            state: Arc::new(Mutex::new(NodeAnnotationState::New)),
            explanation: Arc::new(Mutex::new(None)),
        })
    }

    pub fn state(&self) -> NodeAnnotationState {
        (*self.state.lock()).clone()
    }

    pub fn id(&self) -> uuid::Uuid {
        self.tracking_area.id
    }

    pub fn codeblock(&self) -> &AnnotationCodeBlock {
        &self.node_code_block
    }

    pub fn update_visualization(&self, text: &XcodeText) -> Result<(), DocsGenerationError> {
        // 1. Get the coordinates of the CodeDocumentFrame
        let code_document_frame_origin = get_code_document_frame_properties(&GetVia::Current)
            .map_err(|e| DocsGenerationError::GenericError(e.into()))?
            .dimensions
            .origin;

        // 2. Get the annotation bounds, naturally in global coordinates
        let (annotation_rect_opt, codeblock_bounds) =
            Self::calculate_annotation_bounds(text, &self.node_code_block)?;

        // 3. Publish annotation_rect and codeblock_rect to frontend, this time in LOCAL coordinates. Even if empty, publish to remove ghosts from previous messages.
        NodeAnnotationEvent::UpdateNodeAnnotation(UpdateNodeAnnotationMessage {
            id: self.tracking_area.id,
            annotation_icon: annotation_rect_opt
                .map(|rect| rect.to_local(&code_document_frame_origin)),
            annotation_codeblock: codeblock_bounds
                .map(|rect| rect.to_local(&code_document_frame_origin)),
            window_uid: self.tracking_area.window_uid,
        })
        .publish_to_tauri(&app_handle());

        Ok(())
    }

    pub fn prepare_docs_insertion_position(
        &self,
        text_content: &XcodeText,
    ) -> Result<(), DocsGenerationError> {
        let (docs_insertion_index, _) = compute_docs_insertion_point_and_indentation(
            &text_content,
            self.codeblock().first_char_pos.row,
        )?;

        *NODE_EXPLANATION_CURRENT_INSERTION_POINT.lock() = docs_insertion_index;
        Ok(())
    }

    pub fn generate_node_explanation(&self) -> Result<(), DocsGenerationError> {
        let mut state = (self.state).lock();
        *state = NodeAnnotationState::FetchingExplanation;

        EventRuleExecutionState::NodeExplanationStarted().publish_to_tauri(&app_handle());

        tauri::async_runtime::spawn({
            let state = self.state.clone();
            let explanation = self.explanation.clone();
            let tracking_area = self.tracking_area.clone();
            let name = self.node_code_block.name.clone();

            let codeblock = self.node_code_block.clone();
            let complexity = codeblock.func_complexity_todo;
            async move {
                let response = fetch_node_explanation(codeblock).await;

                if let Ok(response) = response {
                    (*explanation.lock()) = Some(response.clone());
                    let node_explanation_msg = UpdateNodeExplanationMessage {
                        explanation: response,
                        name,
                        complexity,
                    };
                    // Notify the frontend that loading has finished
                    NodeExplanationEvent::UpdateNodeExplanation(node_explanation_msg.clone())
                        .publish_to_tauri(&app_handle());

                    EventRuleExecutionState::NodeExplanationFetched(
                        NodeExplanationFetchedMessage {
                            window_uid: tracking_area.window_uid,
                            annotation_frame: Some(tracking_area.rectangle),
                        },
                    )
                    .publish_to_tauri(&app_handle());
                    debug!(explanation=?node_explanation_msg, "Node explanation fetched");
                } else {
                    EventRuleExecutionState::NodeExplanationFailed()
                        .publish_to_tauri(&app_handle());
                    debug!("NodeExplanationFailed");
                    (*explanation.lock()) = None;
                }
                (*state.lock()) = NodeAnnotationState::Finished;
            }
        });

        Ok(())
    }

    pub fn create_tracking_area(
        text: &XcodeText,
        code_block: &AnnotationCodeBlock,
        window_uid: EditorWindowUid,
    ) -> Result<TrackingArea, DocsGenerationError> {
        // When we create the annotation, we need to compute the bounds for the frontend so it knows where to display the annotation
        // and we need to create a tracking area which the tracking area manager takes care of. The tracking area manager uses GLOBAL coordinates
        // whereas the frontend uses LOCAL coordinates; local to the CodeDocumentFrame.

        // 1. Get the annotation bounds, naturally in global coordinates
        let (annotation_rect_opt, _) = Self::calculate_annotation_bounds(text, code_block)?;

        let tracking_area = TrackingArea {
            id: uuid::Uuid::new_v4(),
            window_uid,
            rectangle: annotation_rect_opt.map_or(LogicalFrame::default(), |rect| rect),
            event_subscriptions: TrackingEventSubscription::TrackingEventTypes(vec![
                TrackingEventType::MouseClicked,
                TrackingEventType::MouseEntered,
                TrackingEventType::MouseExited,
            ]),
            app_window: AppWindow::CodeOverlay,
        };

        // 3. Publish to the tracking area manager with its original GLOBAL coordinates
        EventTrackingArea::Add(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());

        Ok(tracking_area)
    }

    pub fn update_annotation_tracking_area(
        &mut self,
        text: &XcodeText,
    ) -> Result<(), DocsGenerationError> {
        // 2. Get the local coordinates of the AnnotationSectionFrame
        if let Ok((annotation_rect_opt, _)) =
            Self::calculate_annotation_bounds(text, &self.node_code_block)
        {
            self.tracking_area.rectangle =
                annotation_rect_opt.map_or(LogicalFrame::default(), |rect| rect);

            // Check if the annotation is outside of the viewport and if so, remove the tracking areas
            let viewport = get_viewport_frame(&GetVia::Current).map_err(|_| {
                DocsGenerationError::GenericError(anyhow!("Could not derive textarea dimensions"))
            })?;

            if let Some(annotation_rect) = annotation_rect_opt {
                if !viewport.contains_position(&annotation_rect.top_left())
                    && !viewport.contains_position(&annotation_rect.bottom_left())
                {
                    self.tracking_area.rectangle = LogicalFrame::default();
                }
            }

            EventTrackingArea::Update(vec![self.tracking_area.clone()])
                .publish_to_tauri(&app_handle());

            Ok(())
        } else {
            Err(DocsGenerationError::NodeAnnotationUpdateFailed)
        }
    }

    /// It calculates the bounds of the annotation icon and the codeblock rectangle
    /// The annotation icon is going to be the TrackingArea's rectangle. The codeblock rectangle is
    /// the one that is going to be highlighted.
    fn calculate_annotation_bounds(
        text: &XcodeText,
        code_block: &AnnotationCodeBlock,
    ) -> Result<(Option<LogicalFrame>, Option<LogicalFrame>), DocsGenerationError> {
        // 1. Get viewport dimensions
        let ViewportProperties {
            dimensions: viewport,
            annotation_section,
            code_section: _,
            window_uid: _,
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

impl Drop for NodeAnnotation {
    fn drop(&mut self) {
        NodeAnnotationEvent::RemoveNodeAnnotation(RemoveNodeAnnotationMessage {
            id: self.id(),
            window_uid: self.tracking_area.window_uid,
        })
        .publish_to_tauri(&app_handle());
        EventTrackingArea::Remove(vec![self.id()]).publish_to_tauri(&app_handle());
    }
}
