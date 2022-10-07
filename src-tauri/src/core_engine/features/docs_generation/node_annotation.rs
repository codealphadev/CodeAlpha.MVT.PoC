use std::sync::Arc;

use anyhow::anyhow;
use parking_lot::Mutex;
use tracing::debug;

use crate::{
    app_handle,
    core_engine::{
        annotations_manager::{
            AnnotationJob, AnnotationJobInstructions, AnnotationJobSingleChar, AnnotationJobTrait,
            AnnotationKind,
        },
        events::{
            models::{
                NodeExplanationFetchedMessage, RemoveNodeAnnotationMessage,
                UpdateNodeAnnotationMessage, UpdateNodeExplanationMessage,
            },
            AnnotationEvent, AnnotationManagerEvent, EventRuleExecutionState, NodeExplanationEvent,
        },
        features::FeatureKind,
        syntax_tree::{FunctionParameter, SwiftCodeBlockKind},
        utils::XcodeText,
        EditorWindowUid, TextPosition, TextRange,
    },
    platform::macos::{
        get_bounds_for_TextRange, get_code_document_frame_properties, get_viewport_properties,
        GetVia, ViewportProperties,
    },
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
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
    global_frame: Option<LogicalFrame>,
    id: uuid::Uuid,
    window_uid: EditorWindowUid,
    node_code_block: AnnotationCodeBlock,
    state: Arc<Mutex<NodeAnnotationState>>,
    explanation: Arc<Mutex<Option<NodeExplanation>>>,
}

impl PartialEq for NodeAnnotation {
    fn eq(&self, other: &Self) -> bool {
        self.node_code_block == other.node_code_block
    }
}

impl NodeAnnotation {
    pub fn new(
        code_block: AnnotationCodeBlock,
        text_content: &XcodeText,
        window_uid: EditorWindowUid,
    ) -> Result<Self, DocsGenerationError> {
        let (global_frame, _) = Self::calculate_annotation_bounds(text_content, &code_block)?;

        Ok(Self {
            global_frame,
            id: uuid::Uuid::new_v4(),
            window_uid,
            node_code_block: code_block,
            state: Arc::new(Mutex::new(NodeAnnotationState::New)),
            explanation: Arc::new(Mutex::new(None)),
        })
    }

    pub fn state(&self) -> NodeAnnotationState {
        (*self.state.lock()).clone()
    }

    pub fn id(&self) -> uuid::Uuid {
        self.id
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
        AnnotationEvent::UpdateNodeAnnotation(UpdateNodeAnnotationMessage {
            id: self.id,
            annotation_icon: annotation_rect_opt
                .map(|rect| rect.to_local(&code_document_frame_origin)),
            annotation_codeblock: codeblock_bounds
                .map(|rect| rect.to_local(&code_document_frame_origin)),
            editor_window_uid: self.window_uid,
        })
        .publish_to_tauri();

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
            let window_uid = self.window_uid;
            let global_frame = self.global_frame;
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
                            editor_window_uid: window_uid,
                            annotation_frame: global_frame,
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
            window_uid,
        } = get_viewport_properties(&GetVia::Current).map_err(|_| {
            DocsGenerationError::GenericError(anyhow!("Could not derive textarea dimensions"))
        })?;

        // 2. Calculate the annotation rectangles
        if let (Some(first_char_text_pos), Some(last_char_text_pos)) = (
            code_block.first_char_pos.as_TextIndex(&text),
            code_block.last_char_pos.as_TextIndex(&text),
        ) {
            // test annotation manager
            let first_char = AnnotationJobSingleChar::new(
                &TextRange {
                    index: first_char_text_pos,
                    length: 1,
                },
                AnnotationKind::CodeblockFirstChar,
                AnnotationJobInstructions::default(),
            );

            let last_char = AnnotationJobSingleChar::new(
                &TextRange {
                    index: last_char_text_pos,
                    length: 1,
                },
                AnnotationKind::CodeblockLastChar,
                AnnotationJobInstructions::default(),
            );

            AnnotationManagerEvent::Add((
                uuid::Uuid::new_v4(),
                FeatureKind::DocsGeneration,
                vec![
                    AnnotationJob::SingleChar(first_char),
                    AnnotationJob::SingleChar(last_char),
                ],
                window_uid,
            ))
            .publish_to_tauri();
            println!("AnnotationManagerEvent::Add(job_group).publish_to_tauri();");

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
        AnnotationEvent::RemoveNodeAnnotation(RemoveNodeAnnotationMessage {
            id: self.id,
            editor_window_uid: self.window_uid,
        })
        .publish_to_tauri();
    }
}
