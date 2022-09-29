use std::sync::Arc;

use crate::{
    core_engine::{
        features::{
            complexity_refactoring::check_for_method_extraction,
            feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
            formatter::SwiftFormatError,
        },
        syntax_tree::{SwiftSyntaxTree, SwiftSyntaxTreeError},
        CodeDocument, TextRange,
    },
    platform::macos::replace_text_content,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};
use parking_lot::Mutex;
use tracing::error;
use tree_sitter;
use tree_sitter::Node;

use super::RefactoringOperation;

enum ComplexityRefactoringProcedure {
    ComputeSuggestions,
    PerformOperation,
}

pub struct ComplexityRefactoring {
    is_activated: bool,
    suggestion: Arc<Mutex<Option<RefactoringOperation>>>, // TODO: Make array?
}
const MAX_ALLOWED_COMPLEXITY: isize = 2; // TODO: Raise to be more reasonable?

impl FeatureBase for ComplexityRefactoring {
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
                ComplexityRefactoringProcedure::ComputeSuggestions => {
                    self.compute_suggestions(code_document)
                }
                ComplexityRefactoringProcedure::PerformOperation => {
                    self.perform_operation(code_document)
                }
            }
        } else {
            Ok(())
        }
    }

    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        Ok(())
    }

    fn activate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = true;

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = false;

        Ok(())
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        Ok(())
    }
}

impl ComplexityRefactoring {
    fn compute_suggestions(&mut self, code_document: &CodeDocument) -> Result<(), FeatureError> {
        (*self.suggestion.lock()) = None;

        let selected_text_range = match code_document.selected_text_range() {
            Some(selected_text_range) => selected_text_range,
            None => return Ok(()),
        };

        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    ComplexityRefactoringError::InsufficientContext.into(),
                ))?;

        let selected_function = match get_outermost_selected_functionlike(
            selected_text_range,
            code_document.syntax_tree(),
        )? {
            Some(node) => node,
            None => return Ok(()),
        };

        let node_metadata = code_document
            .syntax_tree()
            .get_node_metadata(&selected_function)
            .map_err(|err| ComplexityRefactoringError::GenericError(err.into()))?;

        if node_metadata.complexities.get_total_complexity() <= MAX_ALLOWED_COMPLEXITY {
            println!("This function is fine");
            return Ok(());
        }
        println!("Problem with complexity in this function");
        //check_for_if_combination(&selected_function, text_content);
        let suggestion_mutex = self.suggestion.clone();
        check_for_method_extraction(
            selected_function,
            text_content,
            &code_document.syntax_tree(),
            &code_document
                .file_path()
                .as_ref()
                .expect("No file path!")
                .clone(), // TODO
            move |refactoring_operation| (*suggestion_mutex.lock()) = Some(refactoring_operation),
        )?;

        Ok(())
    }

    fn perform_operation(&mut self, code_document: &CodeDocument) -> Result<(), FeatureError> {
        let mut operation = self
            .suggestion
            .lock()
            .clone()
            .ok_or(ComplexityRefactoringError::NoOperation)?;

        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    ComplexityRefactoringError::InsufficientContext.into(),
                ))?;

        let mut edited_content = text_content.clone();
        operation.edits.sort_by_key(|e| e.start_index);
        operation.edits.reverse();
        for edit in operation.edits {
            edited_content.replace_range(edit.start_index..edit.end_index, edit.text);
            /* replace_range_with_clipboard_text(
                &app_handle(),
                &GetVia::Current,
                &TextRange {
                    index: edit.start_index,
                    length: edit.end_index - edit.start_index,
                },
                Some(&edit.text.as_string()),
                true,
            )
            .await;*/
        }

        tauri::async_runtime::spawn({
            let file_path = code_document.file_path().clone();
            let selected_text_range = code_document.selected_text_range().clone();
            let text_content = text_content.clone();

            async move {
                let mut command = tauri::api::process::Command::new_sidecar("swiftformat")
                    .map_err(|err| SwiftFormatError::GenericError(err.into()))
                    .unwrap(); // TODO

                // Read .swiftformat settings from file path, even though we use direct stdin input
                if let Some(file_path) = file_path {
                    command = command.args(["--stdinpath".to_string(), file_path]);
                }

                let (mut rx, mut child) = command
                    .spawn()
                    .map_err(|err| SwiftFormatError::GenericError(err.into()))
                    .unwrap(); // TODO: error handling

                child
                    .write(edited_content.as_string().as_bytes())
                    .expect("Failed to write to swiftformat");

                drop(child);
                let mut formatted_content = "".to_string();
                while let Some(event) = rx.recv().await {
                    if let tauri::api::process::CommandEvent::Stdout(line) = event {
                        formatted_content.push_str(&(line + "\n"));
                    }
                }

                match replace_text_content(&text_content, &formatted_content, &selected_text_range)
                    .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        error!(?err, "Error replacing text content")
                    }
                }
                dbg!(formatted_content);
            }
        });

        Ok(())
    }
}
/*
fn check_for_if_combination(node: &Node, text_content: &XcodeText) {
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let query = tree_sitter::Query::new(
        language(),
        r#"
        (if_statement (statements . (if_statement) @inner-if . )) @outer-if
        "#,
    )
    .unwrap(); // TODO
    let text_string = text_content.as_string();
    let matches = query_cursor.matches(&query, *node, text_string.as_bytes());
    let outer_index = query.capture_index_for_name("outer-if").unwrap();

    for each_match in matches {
        let outer_if_capture = each_match
            .captures
            .iter()
            .filter(|c| c.index == outer_index)
            .last()
            .unwrap();
        let node: Node = outer_if_capture.node;
        dbg!(node.id());
        dbg!(node.child_count());
        dbg!(node.named_child_count());
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            dbg!(child.kind());
        }
    }
}
*/
// TODO: Make a diff of all the node changes to be done, and then only apply text changes for those.

impl ComplexityRefactoring {
    pub fn new() -> Self {
        Self {
            suggestion: Arc::new(Mutex::new(None)),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
        }
    }

    fn should_compute(
        &self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Option<ComplexityRefactoringProcedure> {
        match trigger {
            CoreEngineTrigger::OnTextSelectionChange => {
                Some(ComplexityRefactoringProcedure::ComputeSuggestions)
            } // The TextSelectionChange is already triggered on text content change
            CoreEngineTrigger::OnTextContentChange => None,
            CoreEngineTrigger::OnUserCommand => {
                Some(ComplexityRefactoringProcedure::PerformOperation)
            }
            _ => None,
        }
    }
}

fn get_outermost_selected_functionlike<'a>(
    selected_text_range: &'a TextRange,
    syntax_tree: &'a SwiftSyntaxTree,
) -> Result<Option<Node<'a>>, ComplexityRefactoringError> {
    let mut result_node: Option<Node> = None;

    let mut curr_node = match syntax_tree.get_code_node_by_text_range(&selected_text_range) {
        Ok(node) => node,
        Err(SwiftSyntaxTreeError::NoTreesitterNodeFound) => return Ok(None),
        Err(err) => return Err(ComplexityRefactoringError::GenericError(err.into())),
    };

    loop {
        let kind = curr_node.kind();
        if kind == "function_declaration" || kind == "lambda_literal" {
            result_node = Some(curr_node.clone());
        }
        match curr_node.parent() {
            Some(node) => curr_node = node,
            None => break,
        }
    }
    Ok(result_node)
}

#[derive(thiserror::Error, Debug)]
pub enum ComplexityRefactoringError {
    #[error("Insufficient context for complexity refactoring")]
    InsufficientContext,
    #[error("No operation found to apply")]
    NoOperation,
    #[error("Something went wrong when executing this ComplexityRefactoring feature.")]
    GenericError(#[source] anyhow::Error),
}
