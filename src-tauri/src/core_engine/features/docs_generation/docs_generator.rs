use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    core_engine::{
        core_engine::EditorWindowUid,
        events::models::NodeAnnotationClickedMessage,
        features::{
            CoreEngineTrigger, FeatureBase, FeatureError, FeatureKind, FeatureProcedure,
            UserCommand,
        },
        syntax_tree::{SwiftCodeBlock, SwiftCodeBlockBase, SwiftSyntaxTree},
        utils::XcodeText,
        CodeDocument, TextPosition, TextRange, XcodeChar,
    },
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

use super::{
    node_annotation::{AnnotationCodeBlock, NodeAnnotationState},
    NodeAnnotation,
};

#[derive(thiserror::Error, Debug)]
pub enum DocsGenerationError {
    #[error("The docs generator does not have sufficient context to proceed.")]
    MissingContext,
    #[error("Something went wrong when executing the DocsGenerator feature.")]
    GenericError(#[source] anyhow::Error),
}

#[derive(Debug)]
enum DocsGenComputeProcedure {
    CreateNewNodeAnnotation,
    FetchNodeExplanation(NodeAnnotationClickedMessage),
}

pub struct DocsGenerator {
    node_annotations: HashMap<EditorWindowUid, NodeAnnotation>,
    is_activated: bool,
    compute_results_updated: bool,
}

impl FeatureBase for DocsGenerator {
    fn compute_short_running(
        &mut self,
        code_document: CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        let no_annotation_is_running =
            !self.is_docs_gen_task_running(&code_document.editor_window_props().window_uid);

        if let Some(procedure) = Self::determine_procedure(trigger, Some(no_annotation_is_running))
        {
            match procedure {
                DocsGenComputeProcedure::FetchNodeExplanation(msg) => {
                    self.procedure_fetch_node_explanation(&code_document, msg)?;
                }
                DocsGenComputeProcedure::CreateNewNodeAnnotation => {
                    if self
                        .procedure_create_new_annotation(&code_document)
                        .is_err()
                    {
                        self.node_annotations
                            .remove(&code_document.editor_window_props().window_uid);
                    }
                }
            }
        }

        Ok(())
    }

    fn compute_long_running(
        &mut self,
        _code_document: CodeDocument,
        _trigger: &CoreEngineTrigger,
        _execution_id: Option<Uuid>,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        Ok(())
    }

    fn activate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = true;

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = false;
        self.clear_node_annotations();

        Ok(())
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        self.clear_node_annotations();

        Ok(())
    }

    fn kind(&self) -> FeatureKind {
        FeatureKind::DocsGeneration
    }

    fn should_compute(
        _kind: &FeatureKind,
        trigger: &CoreEngineTrigger,
    ) -> Option<FeatureProcedure> {
        match Self::determine_procedure(trigger, None) {
            Some(_) => Some(FeatureProcedure::ShortRunning),
            None => None,
        }
    }

    fn requires_ai(_kind: &FeatureKind, _trigger: &CoreEngineTrigger) -> bool {
        true
    }
}

impl DocsGenerator {
    pub fn new() -> Self {
        Self {
            node_annotations: HashMap::new(),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
            compute_results_updated: false,
        }
    }

    fn determine_procedure(
        trigger: &CoreEngineTrigger,
        no_docs_gen_task_running: Option<bool>,
    ) -> Option<DocsGenComputeProcedure> {
        let no_docs_gen_task_running =
            if let Some(no_docs_gen_task_running) = no_docs_gen_task_running {
                no_docs_gen_task_running
            } else {
                true
            };

        match trigger {
            CoreEngineTrigger::OnTextContentChange | CoreEngineTrigger::OnTextSelectionChange => {
                no_docs_gen_task_running.then(|| DocsGenComputeProcedure::CreateNewNodeAnnotation)
            }
            CoreEngineTrigger::OnUserCommand(cmd) => match cmd {
                UserCommand::NodeAnnotationClicked(msg) => {
                    Some(DocsGenComputeProcedure::FetchNodeExplanation(msg.clone()))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn procedure_create_new_annotation(
        &mut self,
        code_document: &CodeDocument,
    ) -> Result<(), FeatureError> {
        let selected_text_range = match code_document.selected_text_range() {
            Some(range) => range,
            None => {
                return Ok(());
            }
        };

        let text_content = code_document
            .text_content()
            .ok_or(FeatureError::GenericError(
                DocsGenerationError::MissingContext.into(),
            ))?;

        let window_uid = code_document.editor_window_props().window_uid;

        let new_codeblock = Self::derive_codeblock(
            selected_text_range,
            code_document
                .syntax_tree()
                .ok_or(FeatureError::GenericError(
                    DocsGenerationError::MissingContext.into(),
                ))?,
            text_content,
        )?;

        let current_annotation = self.node_annotations.get(&window_uid);
        let did_codeblock_update =
            current_annotation.map_or(true, |current| *current.codeblock() != new_codeblock);

        if current_annotation.is_none() || (current_annotation.is_some() && did_codeblock_update) {
            if self
                .create_node_annotation(new_codeblock, text_content, window_uid)
                .is_err()
            {
                self.node_annotations.remove(&window_uid);
            };
        }

        Ok(())
    }

    fn procedure_fetch_node_explanation(
        &mut self,
        code_document: &CodeDocument,
        msg: NodeAnnotationClickedMessage,
    ) -> Result<(), FeatureError> {
        let text_content = code_document
            .text_content()
            .ok_or(FeatureError::GenericError(
                DocsGenerationError::MissingContext.into(),
            ))?;
        Ok(
            if let Some(annotation) = self.node_annotations.get_mut(&msg.editor_window_uid) {
                if msg.annotation_id == annotation.id() {
                    annotation.prepare_docs_insertion_position(text_content)?;
                    annotation.generate_node_explanation()?;
                }
            },
        )
    }

    pub fn clear_node_annotations(&mut self) {
        self.node_annotations.clear();
    }

    fn derive_codeblock(
        selected_text_range: &TextRange,
        syntax_tree: &SwiftSyntaxTree,
        text_content: &XcodeText,
    ) -> Result<AnnotationCodeBlock, DocsGenerationError> {
        let codeblock =
            SwiftCodeBlock::from_text_range(syntax_tree, selected_text_range, text_content)
                .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let text = codeblock
            .as_text()
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let context = if let Ok(parent_codeblock) = codeblock.get_parent_code_block() {
            let parent_text = parent_codeblock
                .as_text()
                .map_err(|err| DocsGenerationError::GenericError(err.into()))?;
            Some(parent_text)
        } else {
            None
        };

        let first_char_pos = codeblock.get_first_char_position();
        let last_char_pos = codeblock.get_last_char_position();

        let name = match codeblock {
            SwiftCodeBlock::Function(ref block) => block.get_name(),
            SwiftCodeBlock::Class(ref block) => block.get_name(),
            _ => None,
        };

        let parameters = match codeblock {
            SwiftCodeBlock::Function(ref function) => Some(
                function
                    .get_parameters()
                    .map_err(|err| DocsGenerationError::GenericError(err.into()))?,
            ),
            _ => None,
        };

        let func_complexity = match codeblock {
            SwiftCodeBlock::Function(ref function) => Some(function.get_complexity()),
            _ => None,
        };

        Ok(AnnotationCodeBlock {
            func_complexity_todo: func_complexity,
            name,
            func_parameters_todo: parameters,
            kind: codeblock.get_kind(),
            first_char_pos,
            last_char_pos,
            text,
            context,
        })
    }

    fn create_node_annotation(
        &mut self,
        codeblock: AnnotationCodeBlock,
        text_content: &XcodeText,
        window_uid: EditorWindowUid,
    ) -> Result<(), DocsGenerationError> {
        let new_annotation = NodeAnnotation::new(codeblock, text_content, window_uid)?;

        self.node_annotations.insert(window_uid, new_annotation);
        self.compute_results_updated = true;

        Ok(())
    }

    fn is_docs_gen_task_running(&self, window_uid: &EditorWindowUid) -> bool {
        if let Some(current_annotation) = self.node_annotations.get(&window_uid) {
            current_annotation.state() == NodeAnnotationState::FetchingExplanation
        } else {
            false
        }
    }
}

pub fn compute_docs_insertion_point_and_indentation(
    text_content: &XcodeText,
    insertion_line: usize,
) -> Result<(usize, usize), DocsGenerationError> {
    // split the text into lines

    let line = text_content
        .rows_iter()
        .nth(insertion_line)
        .ok_or(DocsGenerationError::MissingContext)?;

    // count whitespaces in insertion_line until first character
    let mut whitespaces = 0;
    for c_u16 in line {
        if *c_u16 == ' ' as XcodeChar {
            whitespaces += 1;
        } else {
            break;
        }
    }

    let docs_insertion_index = (TextPosition {
        row: insertion_line,
        column: whitespaces,
    })
    .as_TextIndex(&text_content)
    .ok_or(DocsGenerationError::MissingContext)?;

    Ok((docs_insertion_index, whitespaces))
}
