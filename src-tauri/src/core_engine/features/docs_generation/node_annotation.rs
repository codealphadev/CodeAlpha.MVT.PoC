use std::sync::Arc;

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
            models::{NodeExplanationFetchedMessage, UpdateNodeExplanationMessage},
            AnnotationManagerEvent, EventRuleExecutionState, NodeExplanationEvent,
        },
        features::FeatureKind,
        syntax_tree::{FunctionParameter, SwiftCodeBlockKind},
        utils::XcodeText,
        EditorWindowUid, TextPosition, TextRange,
    },
    platform::macos::{get_bounds_for_TextRange, GetVia},
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
    group_id: uuid::Uuid,
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
        node_code_block: AnnotationCodeBlock,
        text_content: &XcodeText,
        window_uid: EditorWindowUid,
    ) -> Result<Self, DocsGenerationError> {
        let group_id = uuid::Uuid::new_v4();

        // Register annotation jobs
        if let (Some(first_char_text_pos), Some(last_char_text_pos)) = (
            node_code_block.first_char_pos.as_TextIndex(&text_content),
            node_code_block.last_char_pos.as_TextIndex(&text_content),
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
                group_id,
                FeatureKind::DocsGeneration,
                vec![
                    AnnotationJob::SingleChar(first_char),
                    AnnotationJob::SingleChar(last_char),
                ],
                window_uid,
            ))
            .publish_to_tauri();
        }

        Ok(Self {
            group_id,
            window_uid,
            node_code_block,
            state: Arc::new(Mutex::new(NodeAnnotationState::New)),
            explanation: Arc::new(Mutex::new(None)),
        })
    }

    pub fn state(&self) -> NodeAnnotationState {
        (*self.state.lock()).clone()
    }

    pub fn id(&self) -> uuid::Uuid {
        self.group_id
    }

    pub fn codeblock(&self) -> &AnnotationCodeBlock {
        &self.node_code_block
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
            // let global_frame = self.global_frame;
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

                    let annotation_frame = if let Ok(ax_bounds_global) = get_bounds_for_TextRange(
                        &TextRange {
                            index: *NODE_EXPLANATION_CURRENT_INSERTION_POINT.lock(),
                            length: 1,
                        },
                        &GetVia::Current,
                    ) {
                        Some(ax_bounds_global)
                    } else {
                        None
                    };

                    EventRuleExecutionState::NodeExplanationFetched(
                        NodeExplanationFetchedMessage {
                            editor_window_uid: window_uid,
                            annotation_frame,
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
}

impl Drop for NodeAnnotation {
    fn drop(&mut self) {
        AnnotationManagerEvent::Remove(self.group_id).publish_to_tauri();
    }
}
