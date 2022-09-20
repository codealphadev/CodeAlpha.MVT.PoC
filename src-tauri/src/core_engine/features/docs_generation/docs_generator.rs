use std::collections::HashMap;

use crate::{
    core_engine::{
        core_engine::WindowUid,
        features::{CoreEngineTrigger, FeatureBase, FeatureError},
        syntax_tree::{SwiftCodeBlock, SwiftCodeBlockBase, SwiftSyntaxTree},
        utils::XcodeText,
        CodeDocument, TextPosition, TextRange, XcodeChar,
    },
    window_controls::models::TrackingAreaClickedMessage,
    CORE_ENGINE_ACTIVE_AT_STARTUP, NODE_EXPLAINATION_CURRENT_INSERTION_POINT,
};

use super::{
    node_annotation::{CodeBlock, NodeAnnotationState},
    NodeAnnotation,
};

#[derive(thiserror::Error, Debug)]
pub enum DocsGenerationError {
    #[error("The docs generator does not have sufficient context to proceed.")]
    MissingContext,
    #[error(
        "Updating a node annotation has failed. DocsGenManager is advised to drop the annotation."
    )]
    NodeAnnotationUpdateFailed,
    #[error("Something went wrong when executing the DocsGenerator feature.")]
    GenericError(#[source] anyhow::Error),
}

enum DocsGenComputeProcedure {
    CreateNewNodeAnnotation,
    UpdateExistingNodeAnnotation,
    FetchNodeExplanation(TrackingAreaClickedMessage),
}

pub struct DocsGenerator {
    node_annotations: HashMap<WindowUid, NodeAnnotation>,
    is_activated: bool,
    compute_results_updated: bool,
}

impl FeatureBase for DocsGenerator {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        if let Some(procedure) = self.should_compute(code_document, trigger) {
            match procedure {
                DocsGenComputeProcedure::UpdateExistingNodeAnnotation => {
                    self.procedure_update_existing_node_annotation(code_document)?;
                }
                DocsGenComputeProcedure::FetchNodeExplanation(msg) => {
                    self.procedure_fetch_node_explanation(msg)?;
                }
                DocsGenComputeProcedure::CreateNewNodeAnnotation => {
                    self.procedure_create_new_annotation(code_document)?;
                }
            }
        }

        return Ok(());
    }

    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated || !self.should_update_visualization(trigger) {
            return Ok(());
        }

        let window_uid = code_document.editor_window_props().window_uid;

        if let Some(text_content) = code_document.text_content() {
            if let Some(annotation) = self.node_annotations.get_mut(&window_uid) {
                // Visualize the existing annotation. If this fails, we remove the annotation from the map.
                if let Err(error) = annotation.update_visualization(text_content) {
                    self.node_annotations.remove(&window_uid);
                    return Err(FeatureError::GenericError(error.into()));
                }
            }
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
}

impl DocsGenerator {
    pub fn new() -> Self {
        Self {
            node_annotations: HashMap::new(),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
            compute_results_updated: false,
        }
    }

    fn should_update_visualization(&mut self, trigger: &CoreEngineTrigger) -> bool {
        match trigger {
            CoreEngineTrigger::OnTextContentChange => true,
            CoreEngineTrigger::OnTextSelectionChange => {
                if self.compute_results_updated {
                    // Reset the flag.
                    self.compute_results_updated = false;
                    true
                } else {
                    false
                }
            }
            CoreEngineTrigger::OnVisibleTextRangeChange => false,
            CoreEngineTrigger::OnViewportMove => true,
            CoreEngineTrigger::OnViewportDimensionsChange => true,

            _ => false,
        }
    }

    fn should_compute(
        &self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Option<DocsGenComputeProcedure> {
        let no_annotation_is_running =
            !self.is_docs_gen_task_running(&code_document.editor_window_props().window_uid);

        match trigger {
            CoreEngineTrigger::OnTextContentChange => {
                no_annotation_is_running.then(|| DocsGenComputeProcedure::CreateNewNodeAnnotation)
            }
            CoreEngineTrigger::OnTextSelectionChange => {
                no_annotation_is_running.then(|| DocsGenComputeProcedure::CreateNewNodeAnnotation)
            }
            CoreEngineTrigger::OnTrackingAreaClicked(msg) => {
                Some(DocsGenComputeProcedure::FetchNodeExplanation(msg.clone()))
            }
            CoreEngineTrigger::OnVisibleTextRangeChange => {
                Some(DocsGenComputeProcedure::UpdateExistingNodeAnnotation)
            }
            CoreEngineTrigger::OnViewportMove => {
                Some(DocsGenComputeProcedure::UpdateExistingNodeAnnotation)
            }
            CoreEngineTrigger::OnViewportDimensionsChange => {
                Some(DocsGenComputeProcedure::UpdateExistingNodeAnnotation)
            }
            CoreEngineTrigger::OnShortcutPressed(_) => None,
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

        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    DocsGenerationError::MissingContext.into(),
                ))?;

        let window_uid = code_document.editor_window_props().window_uid;

        let new_codeblock =
            Self::derive_codeblock(selected_text_range, code_document.syntax_tree())?;

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
        msg: TrackingAreaClickedMessage,
    ) -> Result<(), FeatureError> {
        Ok(
            if let Some(annotation) = self.node_annotations.get_mut(&msg.window_uid) {
                if msg.id == annotation.id() {
                    annotation.generate_node_explanation()?;
                }
            },
        )
    }

    fn procedure_update_existing_node_annotation(
        &mut self,
        code_document: &CodeDocument,
    ) -> Result<(), FeatureError> {
        if let Some(annotation) = self
            .node_annotations
            .get_mut(&code_document.editor_window_props().window_uid)
        {
            let text_content =
                code_document
                    .text_content()
                    .as_ref()
                    .ok_or(FeatureError::GenericError(
                        DocsGenerationError::MissingContext.into(),
                    ))?;

            if let Err(error) = annotation.update_annotation_tracking_area(text_content) {
                self.node_annotations
                    .remove(&code_document.editor_window_props().window_uid);
                return Err(FeatureError::GenericError(error.into()));
            }
        }

        Ok(())
    }

    pub fn clear_node_annotations(&mut self) {
        self.node_annotations.clear();
    }

    fn derive_codeblock(
        selected_text_range: &TextRange,
        syntax_tree: &SwiftSyntaxTree,
    ) -> Result<CodeBlock, DocsGenerationError> {
        let codeblock: SwiftCodeBlock = syntax_tree
            .get_selected_codeblock_node(&selected_text_range)
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let text = codeblock
            .as_text()
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

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

        Ok(CodeBlock {
            func_complexity_todo: func_complexity,
            name,
            func_parameters_todo: parameters,
            kind: codeblock.get_kind(),
            first_char_pos,
            last_char_pos,
            text,
        })
    }

    fn create_node_annotation(
        &mut self,
        codeblock: CodeBlock,
        text_content: &XcodeText,
        window_uid: WindowUid,
    ) -> Result<(), DocsGenerationError> {
        let (docs_insertion_index, _) = compute_docs_insertion_point_and_indentation(
            text_content,
            codeblock.first_char_pos.row,
        )?;

        *NODE_EXPLAINATION_CURRENT_INSERTION_POINT.lock() = docs_insertion_index;

        let new_annotation = NodeAnnotation::new(codeblock, text_content, window_uid)?;

        self.node_annotations.insert(window_uid, new_annotation);
        self.compute_results_updated = true;

        Ok(())
    }

    fn is_docs_gen_task_running(&self, window_uid: &WindowUid) -> bool {
        if let Some(current_annotation) = self.node_annotations.get(&window_uid) {
            current_annotation.state() == NodeAnnotationState::FetchingExplanation
        } else {
            false
        }
    }
}

fn compute_docs_insertion_point_and_indentation(
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
