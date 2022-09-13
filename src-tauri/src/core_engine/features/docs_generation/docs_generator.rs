use std::collections::HashMap;

use crate::{
    core_engine::{
        core_engine::WindowUid,
        features::{CoreEngineTrigger, FeatureBase, FeatureError},
        syntax_tree::{SwiftCodeBlock, SwiftSyntaxTree},
        utils::{XcodeChar, XcodeText},
        CodeDocument, TextPosition, TextRange,
    },
    window_controls::models::TrackingAreaClickedMessage,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

use super::{docs_generation_task::CodeBlock, DocsGenerationTask, DocsGenerationTaskState};

#[derive(thiserror::Error, Debug)]
pub enum DocsGenerationError {
    #[error("The docs generator does not have sufficient context to proceed.")]
    MissingContext,
    #[error(
        "Updating a docs generation task has failed. DocsGenManager is advised to drop the task."
    )]
    DocsGenTaskUpdateFailed,
    #[error("Something went wrong when executing the DocsGenerator feature.")]
    GenericError(#[source] anyhow::Error),
}

type DocsIndentation = usize;
type DocsInsertionIndex = usize;
enum DocsGenComputeProcedure {
    CreateNewTask,
    UpdateExistingTask,
    GenerateDocs(TrackingAreaClickedMessage),
}

pub struct DocsGenerator {
    docs_generation_tasks: HashMap<WindowUid, DocsGenerationTask>,
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
                DocsGenComputeProcedure::UpdateExistingTask => {
                    self.procedure_update_existing_task(code_document)?;
                }
                DocsGenComputeProcedure::GenerateDocs(msg) => {
                    self.procedure_generate_docs(msg)?;
                }
                DocsGenComputeProcedure::CreateNewTask => {
                    self.procedure_create_new_task(code_document)?;
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
            if let Some(docs_gen_task) = self.docs_generation_tasks.get_mut(&window_uid) {
                // Visualize the existing task. If this fails, we remove the task from the map.
                if let Err(error) = docs_gen_task.update_visualization(text_content) {
                    self.docs_generation_tasks.remove(&window_uid);
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
        self.clear_docs_generation_taskss();

        Ok(())
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        self.clear_docs_generation_taskss();

        Ok(())
    }
}

impl DocsGenerator {
    pub fn new() -> Self {
        Self {
            docs_generation_tasks: HashMap::new(),
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
        let no_docs_task_is_running =
            !self.is_docs_gen_task_running(&code_document.editor_window_props().window_uid);

        match trigger {
            CoreEngineTrigger::OnTextContentChange => {
                no_docs_task_is_running.then(|| DocsGenComputeProcedure::CreateNewTask)
            }
            CoreEngineTrigger::OnTextSelectionChange => {
                no_docs_task_is_running.then(|| DocsGenComputeProcedure::CreateNewTask)
            }
            CoreEngineTrigger::OnTrackingAreaClicked(msg) => {
                Some(DocsGenComputeProcedure::GenerateDocs(msg.clone()))
            }
            CoreEngineTrigger::OnVisibleTextRangeChange => {
                Some(DocsGenComputeProcedure::UpdateExistingTask)
            }
            CoreEngineTrigger::OnViewportMove => Some(DocsGenComputeProcedure::UpdateExistingTask),
            CoreEngineTrigger::OnViewportDimensionsChange => {
                Some(DocsGenComputeProcedure::UpdateExistingTask)
            }
            CoreEngineTrigger::OnShortcutPressed(_) => None,
        }
    }

    fn procedure_create_new_task(
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

        let current_task = self.docs_generation_tasks.get(&window_uid);
        let did_codeblock_update =
            current_task.map_or(true, |current| *current.codeblock() != new_codeblock);

        if current_task.is_none() || (current_task.is_some() && did_codeblock_update) {
            if self
                .create_docs_gen_task(new_codeblock, text_content, window_uid)
                .is_err()
            {
                self.docs_generation_tasks.remove(&window_uid);
            };
        }

        Ok(())
    }

    fn procedure_generate_docs(
        &mut self,
        msg: TrackingAreaClickedMessage,
    ) -> Result<(), FeatureError> {
        Ok(
            if let Some(docs_gen_task) = self.docs_generation_tasks.get_mut(&msg.window_uid) {
                if msg.id == docs_gen_task.id() {
                    docs_gen_task.generate_documentation()?;
                }
            },
        )
    }

    fn procedure_update_existing_task(
        &mut self,
        code_document: &CodeDocument,
    ) -> Result<(), FeatureError> {
        if let Some(docs_gen_task) = self
            .docs_generation_tasks
            .get_mut(&code_document.editor_window_props().window_uid)
        {
            let text_content =
                code_document
                    .text_content()
                    .as_ref()
                    .ok_or(FeatureError::GenericError(
                        DocsGenerationError::MissingContext.into(),
                    ))?;

            if let Err(error) = docs_gen_task.update_task_tracking_area(text_content) {
                self.docs_generation_tasks
                    .remove(&code_document.editor_window_props().window_uid);
                return Err(FeatureError::GenericError(error.into()));
            }
        }

        Ok(())
    }

    pub fn clear_docs_generation_taskss(&mut self) {
        self.docs_generation_tasks.clear();
    }

    fn derive_codeblock(
        selected_text_range: &TextRange,
        syntax_tree: &SwiftSyntaxTree,
    ) -> Result<CodeBlock, DocsGenerationError> {
        let codeblock: SwiftCodeBlock = syntax_tree
            .get_selected_codeblock_node(&selected_text_range)
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let text = codeblock
            .get_codeblock_text()
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let first_char_pos = codeblock.get_first_char_position();
        let last_char_pos = codeblock.get_last_char_position();

        Ok(CodeBlock {
            first_char_pos,
            last_char_pos,
            text,
        })
    }

    fn create_docs_gen_task(
        &mut self,
        codeblock: CodeBlock,
        text_content: &XcodeText,
        window_uid: WindowUid,
    ) -> Result<(), DocsGenerationError> {
        let (docs_insertion_index, _) = self.compute_docs_insertion_point_and_indentation(
            text_content,
            codeblock.first_char_pos.row,
        )?;

        let new_task = DocsGenerationTask::new(
            codeblock,
            text_content,
            TextRange {
                index: docs_insertion_index,
                length: 0,
            },
            window_uid,
        )?;

        self.docs_generation_tasks.insert(window_uid, new_task);
        self.compute_results_updated = true;

        Ok(())
    }

    fn compute_docs_insertion_point_and_indentation(
        &self,
        text_content: &XcodeText,
        insertion_line: usize,
    ) -> Result<(DocsInsertionIndex, DocsIndentation), DocsGenerationError> {
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

    fn is_docs_gen_task_running(&self, window_uid: &WindowUid) -> bool {
        if let Some(current_task) = self.docs_generation_tasks.get(&window_uid) {
            match current_task.task_state() {
                DocsGenerationTaskState::Processing => true,
                _ => false,
            }
        } else {
            false
        }
    }
}
