use std::sync::Arc;

use anyhow::anyhow;
use parking_lot::Mutex;
use tracing::error;

use crate::{
    app_handle,
    core_engine::{
        events::{
            models::{
                NodeExplanationFetchedMessage, RemoveNodeAnnotationMessage,
                UpdateNodeAnnotationMessage,
            },
            EventDocsGeneration, EventRuleExecutionState,
        },
        syntax_tree::SwiftCodeBlockKind,
        utils::XcodeText,
        TextPosition, TextRange, WindowUid,
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
};

use super::{docs_generator::DocsGenerationError, fetch_node_explanation, NodeExplanation};

#[derive(Clone, Debug, PartialEq)]
pub enum NodeAnnotationState {
    New,
    FetchingExplanation,
    Finished,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub name: Option<String>,
    pub parameter_names: Option<Vec<String>>, // TODO: COD-320 Majorly refactor CodeBlock. Not ok to allow incompatible kind and parameters etc.
    pub first_char_pos: TextPosition,
    pub last_char_pos: TextPosition,
    pub kind: SwiftCodeBlockKind,
    pub text: XcodeText,
}

#[derive(Debug, Clone)]
pub struct NodeAnnotation {
    tracking_area: TrackingArea,
    codeblock: CodeBlock,
    state: Arc<Mutex<NodeAnnotationState>>,
    explanation: Arc<Mutex<Option<NodeExplanation>>>,
}

impl PartialEq for NodeAnnotation {
    fn eq(&self, other: &Self) -> bool {
        self.tracking_area.eq_props(&other.tracking_area) && self.codeblock == other.codeblock
    }
}

impl NodeAnnotation {
    pub fn new(
        codeblock: CodeBlock,
        text_content: &XcodeText,
        window_uid: WindowUid,
    ) -> Result<Self, DocsGenerationError> {
        let tracking_area = Self::create_tracking_area(text_content, &codeblock, window_uid)?;

        Ok(Self {
            tracking_area,
            codeblock,
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

    pub fn codeblock(&self) -> &CodeBlock {
        &self.codeblock
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
        EventDocsGeneration::UpdateNodeAnnotation(UpdateNodeAnnotationMessage {
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

    pub fn fetch_node_explanation(&self) -> Result<(), DocsGenerationError> {
        let mut state = (self.state).lock();
        *state = NodeAnnotationState::FetchingExplanation;

        EventRuleExecutionState::DocsGenerationStarted().publish_to_tauri(&app_handle());

        tauri::async_runtime::spawn({
            let state = self.state.clone();
            let explanation = self.explanation.clone();
            let tracking_area = self.tracking_area.clone();
            let name = self.codeblock.name.clone();

            let codeblock = self.codeblock.clone();
            async move {
                let response = fetch_node_explanation(&codeblock, None).await;

                if let Ok(response) = response {
                    (*explanation.lock()) = Some(response.clone());
                    // Notify the frontend that loading has finished
                    EventRuleExecutionState::DocsGenerationFinished()
                        .publish_to_tauri(&app_handle());

                    let message = NodeExplanationFetchedMessage {
                        window_uid: tracking_area.window_uid,
                        annotation_frame: Some(*tracking_area.rectangles.first().unwrap()),
                        explanation: response,
                        name,
                    };
                    EventDocsGeneration::NodeExplanationFetched(message.clone())
                        .publish_to_tauri(&app_handle());
                    EventRuleExecutionState::NodeExplanationFetched(message)
                        .publish_to_tauri(&app_handle());
                    println!("Finished loading explanation");
                } else {
                    EventRuleExecutionState::DocsGenerationFailed().publish_to_tauri(&app_handle());
                    (*explanation.lock()) = None;
                    error!("Fetching node explanation failed");
                }
                (*state.lock()) = NodeAnnotationState::Finished;
            }
        });

        Ok(())
    }

    pub fn create_tracking_area(
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
            Err(DocsGenerationError::NodeAnnotationUpdateFailed)
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
        EventDocsGeneration::RemoveNodeAnnotation(RemoveNodeAnnotationMessage {
            id: self.id(),
            window_uid: self.tracking_area.window_uid,
        })
        .publish_to_tauri(&app_handle());
        EventTrackingArea::Remove(vec![self.id()]).publish_to_tauri(&app_handle());
    }
}
