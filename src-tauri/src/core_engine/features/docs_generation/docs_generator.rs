use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::{
    core_engine::{rules::TextRange, syntax_tree::SwiftSyntaxTree},
    utils::messaging::ChannelList,
    window_controls::events::EventWindowControls,
};

use super::docs_generation_task::DocsGenerationTask;

pub struct DocsGenerator {
    swift_syntax_tree: SwiftSyntaxTree,
    text_content: Option<String>,
    selected_text_range: Option<TextRange>,
    docs_generation_task: Option<DocsGenerationTask>,
    window_pid: i32,
}

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

    pub fn update_content(&mut self, text_content: &String) {
        if self.swift_syntax_tree.parse(text_content) {
            self.text_content = Some(text_content.to_owned());

            // Create a new DocsGenerationTask if there is no task running.
            // We create a new one because the text has changed and code annotation might need to be recomputed
            if !self.is_docs_gen_task_running() {
                let mut newly_created_docs_task = None;
                if let Some(selected_text_range) = self.selected_text_range {
                    if let Some(codeblock) = self
                        .swift_syntax_tree
                        .get_selected_codeblock_node(&selected_text_range)
                    {
                        if let Some(codeblock_text) = codeblock.get_codeblock_text() {
                            let mut new_task = DocsGenerationTask::new(
                                self.window_pid,
                                codeblock.get_first_char_position(),
                                codeblock.get_last_char_position(),
                                codeblock_text,
                            );

                            if new_task.create_code_annotation(text_content) {
                                newly_created_docs_task = Some(new_task);
                            }
                        }
                    }
                }
                self.docs_generation_task = newly_created_docs_task;
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
            let mut newly_created_docs_task = None;
            if let Some(text_content) = self.text_content.as_ref() {
                if let Some(codeblock) = self
                    .swift_syntax_tree
                    .get_selected_codeblock_node(&selected_text_range)
                {
                    if let Some(codeblock_text) = codeblock.get_codeblock_text() {
                        let mut new_task = DocsGenerationTask::new(
                            self.window_pid,
                            codeblock.get_first_char_position(),
                            codeblock.get_last_char_position(),
                            codeblock_text,
                        );

                        if new_task.create_code_annotation(text_content) {
                            newly_created_docs_task = Some(new_task);
                        }
                    }
                }
            }

            self.docs_generation_task = newly_created_docs_task;
        }
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
