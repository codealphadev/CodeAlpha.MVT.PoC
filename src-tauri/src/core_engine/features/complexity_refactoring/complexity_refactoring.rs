use std::{collections::HashMap, sync::Arc};

use crate::{
    app_handle,
    core_engine::{
        events::{
            models::{RemoveSuggestionMessage, UpdateSuggestionMessage},
            SuggestionEvent,
        },
        features::{
            complexity_refactoring::{
                check_for_method_extraction, method_extraction::do_method_extraction,
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
use uuid::Uuid;

use super::{NodeSlice, SerializedNodeSlice};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub enum RefactoringKind {
    MethodExtraction,
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

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub struct FERefactoringSuggestion {
    pub id: uuid::Uuid,
    pub new_text_content_string: String, // TODO: Future?
    pub old_text_content_string: String, // TODO: Future?
    pub new_complexity: isize,
    pub prev_complexity: isize,
    pub main_function_name: Option<String>, // TODO: Should it be an option?
}

#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub id: uuid::Uuid,
    pub new_text_content_string: Option<String>, // TODO: Future? // TODO: Use Xcode text - pasting is probably broken with utf 16 :(
    pub old_text_content_string: Option<String>, // TODO: Future?
    pub new_complexity: isize,
    pub prev_complexity: isize,
    pub main_function_name: Option<String>,
    pub serialized_slice: SerializedNodeSlice,
}

pub fn map_refactoring_suggestion_to_fe_refactoring_suggestion(
    suggestion: RefactoringSuggestion,
) -> Result<FERefactoringSuggestion, ComplexityRefactoringError> {
    Ok(FERefactoringSuggestion {
        id: suggestion.id,
        new_text_content_string: suggestion
            .new_text_content_string
            .ok_or(ComplexityRefactoringError::SuggestionIncomplete)?,
        old_text_content_string: suggestion
            .old_text_content_string
            .ok_or(ComplexityRefactoringError::SuggestionIncomplete)?,
        new_complexity: suggestion.new_complexity,
        prev_complexity: suggestion.prev_complexity,
        main_function_name: suggestion.main_function_name,
    })
}
pub struct ComplexityRefactoring {
    is_activated: bool,
    suggestions: Arc<Mutex<HashMap<String, Vec<RefactoringSuggestion>>>>, // Map function s-exp to suggestions
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
                ComplexityRefactoringProcedure::PerformOperation(id) => self
                    .perform_operation(code_document, id)
                    .map_err(|e| e.into()),
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
        let text_content = code_document
            .text_content()
            .as_ref()
            .ok_or(FeatureError::GenericError(
                ComplexityRefactoringError::InsufficientContext.into(),
            ))?
            .clone();

        let top_level_functions =
            SwiftFunction::get_top_level_functions(code_document.syntax_tree(), &text_content)
                .map_err(|err| ComplexityRefactoringError::GenericError(err.into()))?;

        let file_path = code_document.file_path().clone();

        let suggestions_cache_arc = self.suggestions.clone();
        let mut suggestions_cache = self.suggestions.lock();

        let mut s_exps = vec![];
        for function in top_level_functions {
            let function_s_exp = function.props.node.to_sexp();
            let suggestions_opt = suggestions_cache.get(&function_s_exp);
            let suggestions = match suggestions_opt {
                Some(suggestions) => suggestions.clone(),
                None => {
                    let prev_complexity = function.get_complexity();
                    if prev_complexity <= MAX_ALLOWED_COMPLEXITY {
                        continue;
                    }
                    let (slice, new_complexity) = match check_for_method_extraction(
                        &function,
                        &text_content,
                        &code_document.syntax_tree(),
                    )? {
                        Some(result) => result,
                        None => continue,
                    };
                    let new_suggestions = vec![RefactoringSuggestion {
                        id: uuid::Uuid::new_v4(),
                        serialized_slice: slice.serialize(function.props.node),
                        main_function_name: function.get_name(),
                        new_complexity,
                        prev_complexity,
                        old_text_content_string: None,
                        new_text_content_string: None,
                    }];

                    suggestions_cache.insert(function_s_exp.clone(), new_suggestions.clone());
                    new_suggestions
                }
            };
            for suggestion in suggestions {
                let slice =
                    NodeSlice::deserialize(&suggestion.serialized_slice, function.props.node);
                let binded_text_content = text_content.clone();
                let binded_file_path = file_path.clone();
                let binded_s_exp = function_s_exp.clone();
                let binded_suggestions_cache_arc = suggestions_cache_arc.clone();
                do_method_extraction(
                    slice,
                    move |edits: Vec<Edit>| {
                        Self::update_suggestion_with_text_diff(
                            edits,
                            binded_text_content,
                            binded_suggestions_cache_arc,
                            binded_file_path, //TODO,
                            binded_s_exp,
                            suggestion.id,
                        )
                    },
                    &text_content.clone(),
                    &file_path.clone().unwrap(), // TODO
                );
            }
            s_exps.push(function_s_exp);
        }
        suggestions_cache.retain(|k, _| s_exps.contains(&k));

        Ok(())
    }

    fn update_suggestion_with_text_diff(
        edits: Vec<Edit>,
        text_content: XcodeText,
        suggestions_cache: Arc<Mutex<HashMap<String, Vec<RefactoringSuggestion>>>>,
        file_path: Option<String>,
        function_s_exp: String,
        suggestion_id: Uuid,
    ) {
        tauri::async_runtime::spawn(async move {
            let (old_content, new_content) =
                Self::format_and_apply_edits_to_text_content(edits, text_content, file_path).await;

            let mut suggestions = suggestions_cache.lock();

            if let Some(entry) = suggestions.get_mut(&function_s_exp) {
                if let Some(suggestion) = entry.iter_mut().find(|s| s.id == suggestion_id) {
                    suggestion.new_text_content_string = Some(new_content);
                    suggestion.old_text_content_string = Some(old_content);

                    let fe_suggestion =
                        match map_refactoring_suggestion_to_fe_refactoring_suggestion(
                            suggestion.to_owned(),
                        ) {
                            Err(e) => {
                                error!(?e, "Unable to map suggestion to FE suggestion");
                                return;
                            }
                            Ok(res) => res,
                        };

                    SuggestionEvent::UpdateSuggestion(UpdateSuggestionMessage {
                        suggestion: fe_suggestion,
                    })
                    .publish_to_tauri(&app_handle());
                }
            }
        });
    }

    async fn format_and_apply_edits_to_text_content(
        mut edits: Vec<Edit>,
        text_content: XcodeText,
        file_path: Option<String>,
    ) -> (String, String) {
        let mut edited_content = text_content.clone();

        edits.sort_by_key(|e| e.start_index);
        edits.reverse();

        for edit in edits {
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

        (formatted_old_content, formatted_new_content)
    }

    fn perform_operation(
        &mut self,
        code_document: &CodeDocument,
        suggestion_id: uuid::Uuid,
    ) -> Result<(), ComplexityRefactoringError> {
        let suggestions_cache = self.suggestions.lock().clone();

        let suggestion_to_apply = suggestions_cache
            .values()
            .flatten()
            .find(|f| f.id == suggestion_id)
            .ok_or(ComplexityRefactoringError::SuggestionNotFound)?
            .clone();

        let new_content = suggestion_to_apply
            .new_text_content_string
            .ok_or(ComplexityRefactoringError::SuggestionIncomplete)?;

        let text_content = code_document
            .text_content()
            .as_ref()
            .ok_or(ComplexityRefactoringError::InsufficientContext)?;

        tauri::async_runtime::spawn({
            let selected_text_range = code_document.selected_text_range().clone();
            let text_content = text_content.clone();
            let suggestions_arc = self.suggestions.clone();

            async move {
                match replace_text_content(&text_content, &new_content, &selected_text_range).await
                {
                    Ok(_) => {}
                    Err(err) => {
                        error!(?err, "Error replacing text content");
                        return;
                    }
                }
                let mut suggestions_cache = suggestions_arc.lock();
                if let Some(suggestions_for_function) =
                    suggestions_cache.get_mut(&suggestion_to_apply.serialized_slice.function_sexp)
                {
                    suggestions_for_function.retain(|s| s.id != suggestion_id);
                }

                SuggestionEvent::RemoveSuggestion(RemoveSuggestionMessage { id: suggestion_id })
                    .publish_to_tauri(&app_handle());
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
            suggestions: Arc::new(Mutex::new(HashMap::new())),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
        }
    }

    fn should_compute(
        &self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Option<ComplexityRefactoringProcedure> {
        match trigger {
            CoreEngineTrigger::OnTextContentChange => {
                Some(ComplexityRefactoringProcedure::ComputeSuggestions)
            }
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
    #[error("No suggestion found to apply")]
    SuggestionNotFound,
    #[error("Suggestion has incomplete state")]
    SuggestionIncomplete,
    #[error("Something went wrong when executing this ComplexityRefactoring feature.")]
    GenericError(#[source] anyhow::Error),
}
