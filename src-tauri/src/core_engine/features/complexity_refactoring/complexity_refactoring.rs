use std::sync::Arc;

use crate::{
    app_handle,
    core_engine::{
        events::{models::UpdateSuggestionsMessage, SuggestionEvent},
        features::{
            complexity_refactoring::{
                check_for_method_extraction, method_extraction::MethodExtractionOperation,
            },
            feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
        },
        format_code,
        syntax_tree::{
            SwiftCodeBlock, SwiftCodeBlockBase, SwiftFunction, SwiftSyntaxTree,
            SwiftSyntaxTreeError,
        },
        CodeDocument, TextRange, XcodeText,
    },
    platform::macos::replace_text_content,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::error;
use tree_sitter;
use tree_sitter::Node;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub enum RefactoringKind {
    MethodExtraction,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub struct RefactoringOperation {
    pub id: uuid::Uuid,
    pub new_text_content_string: String,
    pub old_text_content_string: String,
    pub new_complexity: isize,
    pub old_complexity: isize,
    pub kind: RefactoringKind,
    pub function_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Edit {
    pub text: XcodeText,
    pub start_index: usize,
    pub end_index: usize,
}

enum ComplexityRefactoringProcedure {
    ComputeSuggestions,
    PerformOperation(uuid::Uuid),
}

pub struct ComplexityRefactoring {
    is_activated: bool,
    suggestions: Arc<Mutex<Vec<RefactoringOperation>>>,
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
                ComplexityRefactoringProcedure::PerformOperation(id) => {
                    self.perform_operation(code_document, id)
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
        self.reset_suggestions();

        let selected_text_range = match code_document.selected_text_range() {
            Some(selected_text_range) => selected_text_range,
            None => return Ok(()),
        };

        let text_content = code_document
            .text_content()
            .as_ref()
            .ok_or(FeatureError::GenericError(
                ComplexityRefactoringError::InsufficientContext.into(),
            ))?
            .clone();

        let binding = text_content.clone();
        let selected_function = match get_outermost_selected_function(
            selected_text_range,
            code_document.syntax_tree(),
            &binding,
        )? {
            Some(f) => f,
            None => return Ok(()),
        };

        if selected_function.get_complexity() <= MAX_ALLOWED_COMPLEXITY {
            println!("This function is fine");
            return Ok(());
        }

        println!("Problem with complexity in this function");
        //check_for_if_combination(&selected_function, text_content);
        let suggestions_arc = self.suggestions.clone();
        let file_path = code_document.file_path().clone();
        let function_name = selected_function.get_name();
        check_for_method_extraction(
            selected_function.props.node,
            &text_content.clone(),
            &code_document.syntax_tree(),
            &code_document
                .file_path()
                .as_ref()
                .expect("No file path!") // TODO
                .clone(),
            move |result: MethodExtractionOperation| {
                Self::set_results_callback(
                    result,
                    text_content.clone(),
                    suggestions_arc.clone(),
                    file_path.clone(),
                    function_name.clone(),
                )
            },
        )?;

        Ok(())
    }

    fn set_results_callback(
        result: MethodExtractionOperation,
        text_content: XcodeText,
        suggestions_arc: Arc<Mutex<Vec<RefactoringOperation>>>,
        file_path: Option<String>,
        function_name: Option<String>,
    ) {
        tauri::async_runtime::spawn(async move {
            let new_suggestion = Self::convert_result_to_refactoring_operation(
                result,
                text_content,
                RefactoringKind::MethodExtraction,
                function_name,
                file_path,
            )
            .await;
            let mut suggestions = suggestions_arc.lock();

            (*suggestions) = vec![new_suggestion]; // TODO: Allow for multiple suggestions at once.
            Self::publish_suggestions(suggestions.clone());
        });
    }

    async fn convert_result_to_refactoring_operation(
        mut result: MethodExtractionOperation,
        text_content: XcodeText,
        kind: RefactoringKind,
        function_name: Option<String>,
        file_path: Option<String>,
    ) -> RefactoringOperation {
        let mut edited_content = text_content.clone();

        result.edits.sort_by_key(|e| e.start_index);
        result.edits.reverse();

        for edit in result.edits {
            // TODO: replace_range for String is more efficient. In general, test feature with emojis and UTF16 etc.
            edited_content.replace_range(edit.start_index..edit.end_index, edit.text);
        }

        let formatted_new_content = match format_code(&edited_content.as_string(), &file_path).await
        {
            Ok(content) => content,
            Err(e) => {
                error!(?e, "Failed to format during refactoring: new content");
                edited_content.as_string()
            }
        };

        let formatted_old_content = match format_code(&text_content.as_string(), &file_path).await {
            Ok(content) => content,
            Err(e) => {
                error!(?e, "Failed to format during refactoring: old content");
                text_content.as_string()
            }
        };

        RefactoringOperation {
            old_text_content_string: formatted_old_content,
            new_text_content_string: formatted_new_content,
            old_complexity: result.prev_complexity,
            new_complexity: result.new_complexity,
            function_name,
            id: uuid::Uuid::new_v4(),
            kind,
        }
    }

    fn perform_operation(
        &mut self,
        code_document: &CodeDocument,
        suggestion_id: uuid::Uuid,
    ) -> Result<(), FeatureError> {
        let operations = self.suggestions.lock().clone();

        let operation = operations
            .into_iter()
            .find(|op| op.id == suggestion_id)
            .ok_or(ComplexityRefactoringError::NoOperation)?;

        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    ComplexityRefactoringError::InsufficientContext.into(),
                ))?;

        tauri::async_runtime::spawn({
            let selected_text_range = code_document.selected_text_range().clone();
            let text_content = text_content.clone();
            let operations_arc = self.suggestions.clone();

            async move {
                // let formatted_content =
                //     match format_code(&operation.new_text_content_string, &file_path).await {
                //         Ok(content) => content,
                //         Err(err) => {
                //             error!(?err, "Error formatting refactored code");
                //             operation.new_text_content_string
                //         }
                //     };

                match replace_text_content(
                    &text_content,
                    &operation.new_text_content_string,
                    &selected_text_range,
                )
                .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        error!(?err, "Error replacing text content");
                        return;
                    }
                }
                let mut operations = operations_arc.lock();
                (*operations).retain(|s| s.id != suggestion_id);
                Self::publish_suggestions(operations.clone());
            }
        });

        Ok(())
    }

    fn reset_suggestions(&mut self) {
        (*self.suggestions.lock()) = vec![];
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
            suggestions: Arc::new(Mutex::new(Vec::new())),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
        }
    }
    fn publish_suggestions(suggestions: Vec<RefactoringOperation>) {
        // TODO: Use proper event syntax
        SuggestionEvent::UpdateSuggestions(UpdateSuggestionsMessage { suggestions })
            .publish_to_tauri(&app_handle())
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
            CoreEngineTrigger::OnUserCommand(msg) => {
                Some(ComplexityRefactoringProcedure::PerformOperation(msg.id))
            }
            _ => None,
        }
    }
}

fn get_outermost_selected_function<'a>(
    selected_text_range: &'a TextRange,
    syntax_tree: &'a SwiftSyntaxTree,
    text_content: &'a XcodeText,
) -> Result<Option<SwiftFunction<'a>>, ComplexityRefactoringError> {
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

    if let Some(node) = result_node {
        let node_metadata = syntax_tree
            .get_node_metadata(&node)
            .map_err(|err| ComplexityRefactoringError::GenericError(err.into()))?;

        SwiftFunction::new(syntax_tree, node, node_metadata, &text_content)
            .map_err(|err| ComplexityRefactoringError::GenericError(err.into()))
            .map(|f| match f {
                SwiftCodeBlock::Function(f) => Some(f),
                _ => panic!("Wrong codeblock type"),
            })
    } else {
        Ok(None)
    }
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
