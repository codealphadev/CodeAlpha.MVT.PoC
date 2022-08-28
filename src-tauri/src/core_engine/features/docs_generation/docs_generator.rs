use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    core_engine::{
        core_engine::WindowUid,
        features::{CoreEngineTrigger, FeatureBase, FeatureError},
        syntax_tree::{SwiftCodeBlock, SwiftSyntaxTree},
        utils::{XcodeChar, XcodeText},
        CodeDocument, TextPosition, TextRange,
    },
    utils::messaging::ChannelList,
    window_controls::EventWindowControls,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

use super::docs_generation_task::DocsGenerationTask;

#[derive(thiserror::Error, Debug)]
pub enum DocsGenerationError {
    #[error("The docs generator does not have sufficient context to proceed.")]
    MissingContext,
    #[error("The creation of a docs generation task has failed.")]
    DocsGenTaskCreationFailed,
    #[error("Something went wrong when executing the DocsGenerator feature.")]
    GenericError(#[source] anyhow::Error),
}

type DocsIndentation = usize;
type DocsInsertionIndex = usize;

pub struct DocsGenerator {
    docs_generation_task: HashMap<WindowUid, DocsGenerationTask>,
    is_activated: bool,
}

impl FeatureBase for DocsGenerator {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated || !self.should_compute(trigger) {
            return Ok(());
        }

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
        if !self.is_docs_gen_task_running(&window_uid) {
            if let Ok(new_task) = self.create_docs_gen_task(
                selected_text_range,
                text_content,
                code_document.syntax_tree(),
            ) {
                self.docs_generation_task.insert(window_uid, new_task);
            }
        }

        return Ok(());
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

        if let Some(text_content) = code_document.text_content() {
            if let Some(docs_gen_task) = self
                .docs_generation_task
                .get_mut(&code_document.editor_window_props().window_uid)
            {
                return docs_gen_task
                    .update_code_annotation_position(text_content)
                    .map_err(|err| FeatureError::GenericError(err.into()));
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
        self.clear_docs_generation_tasks();

        Ok(())
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        self.clear_docs_generation_tasks();

        Ok(())
    }
}

impl DocsGenerator {
    pub fn new() -> Self {
        Self {
            docs_generation_task: HashMap::new(),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
        }
    }

    fn should_compute(&self, trigger: &CoreEngineTrigger) -> bool {
        match trigger {
            CoreEngineTrigger::OnTextContentChange => true,
            CoreEngineTrigger::OnTextSelectionChange => true,
            
            _ => false,
        }
    }
    fn should_update_visualization(&self, trigger: &CoreEngineTrigger) -> bool {
        match trigger {
            CoreEngineTrigger::OnTextContentChange => true,
            CoreEngineTrigger::OnTextSelectionChange => true,
            CoreEngineTrigger::OnVisibleTextRangeChange => true,
            CoreEngineTrigger::OnViewportMove =>  true,
            CoreEngineTrigger::OnViewportDimensionsChange => true,

            _ => false,
        }
    }

    pub fn clear_docs_generation_tasks(&mut self) {
        self.docs_generation_task.clear();
    }

    fn create_docs_gen_task(
        &self,
        selected_text_range: &TextRange,
        text_content: &XcodeText,
        window_uid: WindowUid,
        syntax_tree: &SwiftSyntaxTree,
    ) -> Result<DocsGenerationTask, DocsGenerationError> {
        let codeblock: SwiftCodeBlock = syntax_tree
            .get_selected_codeblock_node(&selected_text_range)
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let codeblock_text = codeblock
            .get_codeblock_text()
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let first_char_position = codeblock.get_first_char_position();

        let (docs_insertion_index, docs_indentation) = self
            .compute_docs_insertion_point_and_indentation(text_content, first_char_position.row)?;

        let mut new_task = DocsGenerationTask::new(
            codeblock.get_first_char_position(),
            codeblock.get_last_char_position(),
            TextRange {
                index: docs_insertion_index,
                length: 0,
            },
            codeblock_text,
        );

        if new_task
            .create_code_annotation(text_content, window_uid)
            .is_ok()
        {
            Ok(new_task)
        } else {
            Err(DocsGenerationError::DocsGenTaskCreationFailed)
        }
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

    pub fn start_listener_window_control_events(
        app_handle: &tauri::AppHandle,
        docs_generator: &Arc<Mutex<Self>>,
    ) {
        app_handle.listen_global(ChannelList::EventWindowControls.to_string(), {
            let docs_generator = (docs_generator).clone();
            |msg| {
                let event_window_controls: EventWindowControls =
                    serde_json::from_str(&msg.payload().unwrap()).unwrap();

                let docs_manager = docs_generator.lock();

                match event_window_controls {
                    EventWindowControls::TrackingAreaClicked(msg) => {
                        for docs_generation_task in docs_manager.docs_generation_task {
                            if let Some(task_id) = docs_generation_task.1.id() {
                                if msg.id == task_id {
                                    docs_generation_task.1.generate_documentation();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn is_docs_gen_task_running(&self, window_uid: &WindowUid) -> bool {
        if let Some(current_task) = self.docs_generation_task.get_mut(&window_uid) {
            match current_task.task_state() {
                super::docs_generation_task::DocsGenerationTaskState::Processing => true,
                _ => false,
            }
        } else {
            false
        }
    }
}
