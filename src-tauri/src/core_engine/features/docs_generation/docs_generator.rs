use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    core_engine::{
        core_engine::WindowUid,
        syntax_tree::{SwiftCodeBlock, SwiftSyntaxTree},
        utils::{XcodeChar, XcodeText},
        TextPosition, TextRange,
    },
    utils::messaging::ChannelList,
    window_controls::EventWindowControls,
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

pub struct DocsGenerator {
    swift_syntax_tree: SwiftSyntaxTree,
    text_content: Option<XcodeText>,
    selected_text_range: Option<TextRange>,
    docs_generation_task: HashMap<WindowUid, DocsGenerationTask>,
    window_pid: i32,
}

type DocsIndentation = usize;
type DocsInsertionIndex = usize;

impl DocsGenerator {
    pub fn new(window_pid: i32) -> Self {
        Self {
            swift_syntax_tree: SwiftSyntaxTree::new(),
            text_content: None,
            selected_text_range: None,
            docs_generation_task: HashMap::new(),
            window_pid,
        }
    }

    pub fn clear_docs_generation_tasks(&mut self) {
        self.docs_generation_task.clear();
    }

    fn create_docs_gen_task(
        &self,
        text_content: &XcodeText,
    ) -> Result<DocsGenerationTask, DocsGenerationError> {
        let selected_text_range = &self
            .selected_text_range
            .ok_or(DocsGenerationError::MissingContext)?;

        let codeblock: SwiftCodeBlock = self
            .swift_syntax_tree
            .get_selected_codeblock_node(&selected_text_range)
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let codeblock_text = codeblock
            .get_codeblock_text()
            .map_err(|err| DocsGenerationError::GenericError(err.into()))?;

        let first_char_position = codeblock.get_first_char_position();

        let (docs_insertion_index, docs_indentation) =
            self.compute_docs_insertion_point_and_indentation(first_char_position.row)?;

        let mut new_task = DocsGenerationTask::new(
            self.window_pid,
            codeblock.get_first_char_position(),
            codeblock.get_last_char_position(),
            TextRange {
                index: docs_insertion_index,
                length: 0,
            },
            docs_indentation,
            codeblock_text,
        );

        if new_task.create_code_annotation(text_content).is_ok() {
            Ok(new_task)
        } else {
            Err(DocsGenerationError::DocsGenTaskCreationFailed)
        }
    }

    pub fn update_content(&mut self, text_content: &XcodeText, window_uid: WindowUid) {
        if self.swift_syntax_tree.parse(text_content) {
            self.text_content = Some(text_content.to_owned());

            // Create a new DocsGenerationTask if there is no task running.
            // We create a new one because the text has changed and code annotation might need to be recomputed
            if !self.is_docs_gen_task_running(&window_uid) {
                if let Ok(new_task) = self.create_docs_gen_task(text_content) {
                    self.docs_generation_task.insert(window_uid, new_task);
                }
            } else {
                println!("DocsGenerator: update_content: docs generation task is running");
            }
        }
    }

    pub fn update_selected_text_range(
        &mut self,
        selected_text_range: &TextRange,
        window_uid: WindowUid,
    ) {
        if let Some(text_content) = self.text_content {
            self.selected_text_range = Some(selected_text_range.to_owned());

            // Create a new DocsGenerationTask if there is no task running.
            // We create a new one because the cursor might have moved into a new codeblock. In this case we need to create a new code annotation.
            if !self.is_docs_gen_task_running(&window_uid) {
                if let Ok(new_task) = self.create_docs_gen_task(&text_content) {
                    self.docs_generation_task.insert(window_uid, new_task);
                }
            }
        }
    }

    fn compute_docs_insertion_point_and_indentation(
        &self,
        insertion_line: usize,
    ) -> Result<(DocsInsertionIndex, DocsIndentation), DocsGenerationError> {
        // split the text into lines
        let text = self
            .text_content
            .ok_or(DocsGenerationError::MissingContext)?;

        let line = text
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
        .as_TextIndex(&text)
        .ok_or(DocsGenerationError::MissingContext)?;

        Ok((docs_insertion_index, whitespaces))
    }

    pub fn update_visualization(&mut self, window_uid: &WindowUid) {
        if let Some(text_content) = self.text_content.as_ref() {
            if let Some(docs_gen_task) = self.docs_generation_task.get_mut(&window_uid) {
                docs_gen_task
                    .update_code_annotation_position(text_content)
                    .ok();
            }
        }
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
