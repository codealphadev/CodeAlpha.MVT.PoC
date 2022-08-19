use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::{
    core_engine::{
        rules::{TextPosition, TextRange},
        syntax_tree::{SwiftCodeBlock, SwiftSyntaxTree},
        utils::{XcodeChar, XcodeText},
    },
    utils::messaging::ChannelList,
    window_controls::EventWindowControls,
};

use super::docs_generation_task::DocsGenerationTask;

pub struct DocsGenerator {
    swift_syntax_tree: SwiftSyntaxTree,
    text_content: Option<XcodeText>,
    selected_text_range: Option<TextRange>,
    docs_generation_task: Option<DocsGenerationTask>,
    window_pid: i32,
}

type DocsIndetation = usize;
type DocsInsertionIndex = usize;

impl DocsGenerator {
    pub fn new(window_pid: i32) -> Self {
        Self {
            swift_syntax_tree: SwiftSyntaxTree::new(),
            text_content: None,
            selected_text_range: None,
            docs_generation_task: None,
            window_pid,
        }
    }

    pub fn clear_docs_generation_tasks(&mut self) {
        self.docs_generation_task = None;
    }

    fn create_docs_gen_task(&self, text_content: &XcodeText) -> Option<DocsGenerationTask> {
        let codeblock: SwiftCodeBlock = self
            .swift_syntax_tree
            .get_selected_codeblock_node(&self.selected_text_range?)?;
        let first_char_position = codeblock.get_first_char_position();
        let codeblock_text = codeblock.get_codeblock_text();
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
            return Some(new_task);
        }
        None
    }

    pub fn update_content(&mut self, text_content: &XcodeText) {
        if self.swift_syntax_tree.parse(text_content) {
            self.text_content = Some(text_content.to_owned());

            // Create a new DocsGenerationTask if there is no task running.
            // We create a new one because the text has changed and code annotation might need to be recomputed
            if !self.is_docs_gen_task_running() {
                self.docs_generation_task = self.create_docs_gen_task(text_content)
            } else {
                println!("DocsGenerator: update_content: docs generation task is running");
            }
        }
    }

    pub fn update_selected_text_range(&mut self, selected_text_range: &TextRange) {
        self.selected_text_range = Some(selected_text_range.to_owned());

        // Create a new DocsGenerationTask if there is no task running.
        // We create a new one because the cursor might have moved into a new codeblock. In this case we need to create a new code annotation.
        if !self.is_docs_gen_task_running() {
            if let Some(text_content) = self.text_content.as_ref() {
                self.docs_generation_task = self.create_docs_gen_task(&text_content);
            }
        }
    }

    fn compute_docs_insertion_point_and_indentation(
        &self,
        insertion_line: usize,
    ) -> Option<(DocsInsertionIndex, DocsIndetation)> {
        // split the text into lines
        if let Some(text) = self.text_content.as_ref() {
            if let Some(line) = text.rows_iter().nth(insertion_line) {
                // count whitespaces in insertion_line until first character
                let mut whitespaces = 0;
                for c_u16 in line {
                    if *c_u16 == ' ' as XcodeChar {
                        whitespaces += 1;
                    } else {
                        break;
                    }
                }

                if let Some(docs_insertion_index) = (TextPosition {
                    row: insertion_line,
                    column: whitespaces,
                })
                .as_TextIndex(text)
                {
                    return Some((docs_insertion_index, whitespaces));
                }
            }
        }

        None
    }

    pub fn update_visualization(&mut self) {
        if let Some(text_content) = self.text_content.as_ref() {
            if let Some(docs_gen_task) = &mut self.docs_generation_task {
                docs_gen_task.update_code_annotation_position(text_content);
            }
        }
    }

    pub fn start_listener_window_control_events(
        app_handle: &tauri::AppHandle,
        docs_generator: &Arc<Mutex<Self>>,
    ) {
        let docs_generator_move_copy = (docs_generator).clone();
        app_handle.listen_global(ChannelList::EventWindowControls.to_string(), move |msg| {
            let event_window_controls: EventWindowControls =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            let docs_manager = &mut *(match docs_generator_move_copy.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            });

            match event_window_controls {
                EventWindowControls::TrackingAreaClicked(msg) => {
                    if let Some(docs_generation_task) = &mut docs_manager.docs_generation_task {
                        if let Some(task_id) = docs_generation_task.id() {
                            if msg.id == task_id {
                                docs_generation_task.generate_documentation();
                            }
                        }
                    }
                }
                _ => {}
            }
        });
    }

    fn is_docs_gen_task_running(&self) -> bool {
        if let Some(current_task) = &self.docs_generation_task {
            match current_task.task_state() {
                super::docs_generation_task::DocsGenerationTaskState::Processing => true,
                _ => false,
            }
        } else {
            false
        }
    }
}
