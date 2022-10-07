use super::{NodeSlice, SerializedNodeSlice};
use crate::{
    app_handle,
    core_engine::{
        events::{models::ReplaceSuggestionsMessage, SuggestionEvent},
        features::{
            complexity_refactoring::{
                check_for_method_extraction, method_extraction::do_method_extraction,
            },
            feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
            UserCommand,
        },
        format_code,
        syntax_tree::{SwiftFunction, SwiftSyntaxTree},
        CodeDocument, EditorWindowUid, TextPosition, XcodeText,
    },
    platform::macos::replace_text_content,
    utils::calculate_hash,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tracing::debug;
use tracing::error;
use tracing::warn;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Edit {
    pub text: XcodeText,
    pub start_index: usize,
    pub end_index: usize,
}

enum ComplexityRefactoringProcedure {
    ComputeSuggestions,
    PerformOperation(uuid::Uuid),
    DismissSuggestion(uuid::Uuid),
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub struct FERefactoringSuggestion {
    pub window_uid: usize,
    pub id: uuid::Uuid,
    pub new_text_content_string: Option<String>,
    pub old_text_content_string: Option<String>,
    pub new_complexity: isize,
    pub prev_complexity: isize,
    pub main_function_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub new_text_content_string: Option<String>, // TODO: Use Xcode text - pasting is probably broken with utf 16 :(
    pub old_text_content_string: Option<String>,
    pub new_complexity: isize,
    pub prev_complexity: isize,
    pub main_function_name: Option<String>,
    pub serialized_slice: SerializedNodeSlice,
}

pub fn map_refactoring_suggestion_to_fe_refactoring_suggestion(
    suggestion: RefactoringSuggestion,
    id: Uuid,
    window_uid: usize,
) -> FERefactoringSuggestion {
    FERefactoringSuggestion {
        id,
        new_text_content_string: suggestion.new_text_content_string,
        old_text_content_string: suggestion.old_text_content_string,
        new_complexity: suggestion.new_complexity,
        prev_complexity: suggestion.prev_complexity,
        main_function_name: suggestion.main_function_name,
        window_uid,
    }
}

type SuggestionHash = u64;

pub struct ComplexityRefactoring {
    is_activated: bool,
    suggestions: Arc<Mutex<HashMap<Uuid, RefactoringSuggestion>>>,
    dismissed_suggestions: Arc<Mutex<HashSet<SuggestionHash>>>,
}

const MAX_ALLOWED_COMPLEXITY: isize = 5;

impl FeatureBase for ComplexityRefactoring {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        if let Some(procedure) = self.should_compute(trigger) {
            match procedure {
                ComplexityRefactoringProcedure::ComputeSuggestions => {
                    self.compute_suggestions(code_document).map_err(|e| {
                        self.suggestions.lock().clear();
                        e
                    })
                }
                ComplexityRefactoringProcedure::PerformOperation(id) => self
                    .perform_operation(code_document, id)
                    .map_err(|e| e.into()),
                ComplexityRefactoringProcedure::DismissSuggestion(id) => self
                    .dismiss_suggestion(code_document, id)
                    .map_err(|e| e.into()),
            }
        } else {
            Ok(())
        }
    }

    fn update_visualization(
        &mut self,
        _code_document: &CodeDocument,
        _trigger: &CoreEngineTrigger,
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
        debug!("Computing suggestions for complexity refactoring");
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

        let mut s_exps = vec![];

        let mut suggestions: HashMap<Uuid, RefactoringSuggestion> = HashMap::new();

        for function in top_level_functions {
            s_exps.push(function.props.node.to_sexp());
            suggestions.extend(Self::generate_suggestions_for_function(
                function,
                &text_content,
                &file_path,
                code_document.syntax_tree(),
                self.suggestions.clone(),
                self.dismissed_suggestions.clone(),
                code_document.editor_window_props().window_uid,
            )?);
        }
        let removed_suggestions_count: usize;
        let added_suggestions_count: usize;
        let window_uid = code_document.editor_window_props().window_uid;
        {
            let mut suggestions_cache = self.suggestions.lock();
            let old_suggestions = suggestions_cache.clone();
            (*suggestions_cache) = suggestions.clone();

            added_suggestions_count = suggestions
                .clone()
                .iter()
                .filter(|(id, _)| !old_suggestions.contains_key(&id))
                .count();

            removed_suggestions_count = old_suggestions
                .clone()
                .into_keys()
                .filter(|id| !suggestions.contains_key(id))
                .count();
        }

        if removed_suggestions_count > 0 || added_suggestions_count > 0 {
            Self::publish_to_frontend(suggestions, window_uid);
        }

        Ok(())
    }

    fn publish_to_frontend(suggestions: HashMap<Uuid, RefactoringSuggestion>, window_uid: usize) {
        let fe_suggestions = suggestions
            .into_iter()
            .map(|(id, suggestion)| {
                (
                    id,
                    map_refactoring_suggestion_to_fe_refactoring_suggestion(
                        suggestion, id, window_uid,
                    ),
                )
            })
            .collect::<HashMap<Uuid, FERefactoringSuggestion>>();

        SuggestionEvent::ReplaceSuggestions(ReplaceSuggestionsMessage {
            suggestions: fe_suggestions,
        })
        .publish_to_tauri(&app_handle());
    }

    fn generate_suggestions_for_function(
        function: SwiftFunction,
        text_content: &XcodeText,
        file_path: &Option<String>,
        syntax_tree: &SwiftSyntaxTree,
        suggestions_arc: Arc<Mutex<HashMap<Uuid, RefactoringSuggestion>>>,
        dismissed_suggestions_arc: Arc<Mutex<HashSet<SuggestionHash>>>,
        window_uid: EditorWindowUid,
    ) -> Result<HashMap<Uuid, RefactoringSuggestion>, ComplexityRefactoringError> {
        let old_suggestions = suggestions_arc.lock().clone();

        let suggestions = Self::compute_suggestions_for_function(
            &function,
            &old_suggestions,
            dismissed_suggestions_arc,
            &text_content,
            &syntax_tree,
        )?;

        for (id, suggestion) in suggestions.iter() {
            let slice = NodeSlice::deserialize(&suggestion.serialized_slice, function.props.node)?;

            let range_length = (slice.nodes.last().unwrap().end_byte()
                - slice.nodes.first().unwrap().start_byte())
                / 2; // UTF-16;
            let start_position = TextPosition::from_TSPoint(&slice.nodes[0].start_position());

            let binded_text_content = text_content.clone();
            let binded_text_content_2 = text_content.clone();
            let binded_file_path = file_path.clone();
            let binded_suggestion = suggestion.clone();
            let binded_id: Uuid = *id;
            let binded_suggestions_cache_arc = suggestions_arc.clone();

            // For error reporting
            let serialized_slice = suggestion.serialized_slice.clone();
            let node_kinds = slice.nodes.iter().map(|n| n.kind()).collect::<Vec<_>>();

            let parent_node_kind = slice
                .nodes
                .first()
                .and_then(|n| n.parent())
                .map(|n| n.kind());
            tauri::async_runtime::spawn({
                async move {
                    _ = do_method_extraction(
                        start_position,
                        range_length,
                        move |edits: Vec<Edit>| {
                            Self::update_suggestion_with_formatted_text_diff(
                                binded_id,
                                binded_suggestion,
                                edits,
                                binded_text_content,
                                binded_suggestions_cache_arc,
                                binded_file_path,
                                window_uid,
                            )
                        },
                        &binded_text_content_2,
                    )
                    .await
                    .map_err(|e| match e {
                        ComplexityRefactoringError::LspRejectedRefactoring => {
                            warn!(
                                ?serialized_slice,
                                ?node_kinds,
                                ?parent_node_kind,
                                "LSP rejected refactoring"
                            );
                        }
                        _ => error!(?e, "Failed to perform refactoring"),
                    });
                }
            });
        }
        Ok(suggestions)
    }

    fn compute_suggestions_for_function(
        function: &SwiftFunction,
        old_suggestions: &HashMap<Uuid, RefactoringSuggestion>,
        dismissed_suggestions_arc: Arc<Mutex<HashSet<SuggestionHash>>>,
        text_content: &XcodeText,
        syntax_tree: &SwiftSyntaxTree,
    ) -> Result<HashMap<Uuid, RefactoringSuggestion>, ComplexityRefactoringError> {
        let prev_complexity = function.get_complexity();
        if prev_complexity <= MAX_ALLOWED_COMPLEXITY {
            return Ok(HashMap::new());
        }
        let (serialized_node_slice, new_complexity) =
            match check_for_method_extraction(&function, &text_content, &syntax_tree)? {
                Some(result) => result,
                None => return Ok(HashMap::new()),
            };

        if dismissed_suggestions_arc
            .lock()
            .contains(&calculate_hash(&serialized_node_slice))
        {
            return Ok(HashMap::new());
        }

        let mut new_suggestions = HashMap::new();

        let old_suggestions_with_same_serialization: Vec<(&Uuid, &RefactoringSuggestion)> =
            old_suggestions
                .iter()
                .filter(|&(_, suggestion)| suggestion.serialized_slice == serialized_node_slice)
                .collect::<Vec<_>>();

        // Re-identify ID with previous value to avoid unnecessary removal and addition
        let id = if old_suggestions_with_same_serialization.len() == 1 {
            *old_suggestions_with_same_serialization[0].0
        } else {
            uuid::Uuid::new_v4()
        };

        new_suggestions.insert(
            id,
            RefactoringSuggestion {
                serialized_slice: serialized_node_slice,
                main_function_name: function.get_name(),
                new_complexity,
                prev_complexity,
                old_text_content_string: None,
                new_text_content_string: None,
            },
        );

        Ok(new_suggestions)
    }

    fn update_suggestion(
        id: Uuid,
        updated_suggestion: &RefactoringSuggestion,
        suggestions_cache: Arc<Mutex<HashMap<Uuid, RefactoringSuggestion>>>,
        window_uid: EditorWindowUid,
    ) {
        let mut suggestions = suggestions_cache.lock();
        let suggestion = suggestions.get_mut(&id);

        if let Some(suggestion) = suggestion {
            suggestion.clone_from(updated_suggestion);

            Self::publish_to_frontend(suggestions.clone(), window_uid);
        }
    }

    fn update_suggestion_with_formatted_text_diff(
        id: Uuid,
        mut suggestion: RefactoringSuggestion,
        edits: Vec<Edit>,
        text_content: XcodeText,
        suggestions_cache: Arc<Mutex<HashMap<Uuid, RefactoringSuggestion>>>,
        file_path: Option<String>,
        window_uid: EditorWindowUid,
    ) {
        tauri::async_runtime::spawn(async move {
            let (old_content, new_content) =
                Self::format_and_apply_edits_to_text_content(edits, text_content, file_path).await;

            suggestion.old_text_content_string = Some(old_content);
            suggestion.new_text_content_string = Some(new_content);
            Self::update_suggestion(id, &suggestion, suggestions_cache, window_uid)
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
            .get(&suggestion_id)
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
            let editor_window_uid = code_document.editor_window_props().window_uid;

            async move {
                match replace_text_content(
                    &text_content,
                    &XcodeText::from_str(&new_content),
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
                let mut suggestions = suggestions_arc.lock();
                suggestions.remove(&suggestion_id);

                Self::publish_to_frontend(suggestions.clone(), editor_window_uid);
            }
        });

        Ok(())
    }

    fn dismiss_suggestion(
        &mut self,
        code_document: &CodeDocument,
        suggestion_id: uuid::Uuid,
    ) -> Result<(), ComplexityRefactoringError> {
        let suggestions;
        let hash;
        {
            let mut suggestions_guard = self.suggestions.lock();

            hash = calculate_hash::<SerializedNodeSlice>(
                &suggestions_guard
                    .get(&suggestion_id)
                    .ok_or(ComplexityRefactoringError::SuggestionNotFound)?
                    .serialized_slice,
            );

            suggestions_guard.remove(&suggestion_id);
            suggestions = suggestions_guard.clone();
        }
        self.dismissed_suggestions.lock().insert(hash);

        Self::publish_to_frontend(suggestions, code_document.editor_window_props().window_uid);

        Ok(())
    }
}

impl ComplexityRefactoring {
    pub fn new() -> Self {
        Self {
            suggestions: Arc::new(Mutex::new(HashMap::new())),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
            dismissed_suggestions: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    fn should_compute(
        &self,
        trigger: &CoreEngineTrigger,
    ) -> Option<ComplexityRefactoringProcedure> {
        match trigger {
            CoreEngineTrigger::OnTextContentChange => {
                Some(ComplexityRefactoringProcedure::ComputeSuggestions)
            }
            CoreEngineTrigger::OnUserCommand(UserCommand::PerformRefactoringOperation(msg)) => {
                Some(ComplexityRefactoringProcedure::PerformOperation(msg.id))
            }
            CoreEngineTrigger::OnUserCommand(UserCommand::DismissRefactoringSuggestion(msg)) => {
                Some(ComplexityRefactoringProcedure::DismissSuggestion(msg.id))
            }
            _ => None,
        }
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
    #[error("LSP rejected refactoring operation")]
    LspRejectedRefactoring,
    #[error("Something went wrong when executing this ComplexityRefactoring feature.")]
    GenericError(#[source] anyhow::Error),
}
