use super::{
    procedures, set_annotation_group_for_extraction_and_context,
    ComplexityRefactoringShortRunningProcedure, FERefactoringSuggestion, NodeSlice,
    RefactoringSuggestion, SuggestionHash, SuggestionId, SuggestionState, SuggestionsArcMutex,
    SuggestionsPerWindow,
};
use crate::{
    app_handle,
    core_engine::{
        events::{models::ReplaceSuggestionsMessage, SuggestionEvent},
        features::{
            complexity_refactoring::{
                check_for_method_extraction, method_extraction::get_edits_for_method_extraction,
                remove_annotations_for_suggestions, Edit, SuggestionsMap,
            },
            feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
            FeatureKind, UserCommand,
        },
        format_code, get_index_of_first_difference,
        syntax_tree::{SwiftCodeBlockBase, SwiftFunction, SwiftSyntaxTree},
        CodeDocument, EditorWindowUid, TextPosition, TextRange, XcodeText,
    },
    platform::macos::{replace_text_content, set_selected_text_range, GetVia},
    utils::calculate_hash,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};
use anyhow::anyhow;
use lazy_static::lazy_static;

use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::oneshot;

use tracing::{debug, error, warn};
use uuid::Uuid;

const MAX_ALLOWED_COMPLEXITY: isize = 9;

lazy_static! {
    pub static ref CURRENT_COMPLEXITY_REFACTORING_EXECUTION_ID: Mutex<Option<Uuid>> =
        Mutex::new(None);
}

pub struct ComplexityRefactoring {
    is_activated: bool,
    suggestions_arc: SuggestionsArcMutex,
    dismissed_suggestions: Arc<Mutex<Vec<SuggestionHash>>>,

    cancel_long_running_task_send: Option<oneshot::Sender<&'static ()>>,
}

impl FeatureBase for ComplexityRefactoring {
    fn compute_short_running(
        &mut self,
        code_document: CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        if let Some(procedure) = self.determine_short_running_procedure(trigger) {
            tauri::async_runtime::spawn({
                let dismissed_suggestions = self.dismissed_suggestions.clone();
                let suggestions_arc = self.suggestions_arc.clone();

                async move {
                    match procedure {
                        ComplexityRefactoringShortRunningProcedure::PerformSuggestion(id) => {
                            Self::perform_suggestion(code_document, id, suggestions_arc).await
                        }
                        ComplexityRefactoringShortRunningProcedure::DismissSuggestion(id) => {
                            procedures::dismiss_suggestion(
                                code_document,
                                id,
                                suggestions_arc,
                                dismissed_suggestions,
                            )
                            .await
                        }
                        ComplexityRefactoringShortRunningProcedure::SelectSuggestion(id) => {
                            procedures::select_suggestion(id).await
                        }
                    }
                }
            });
        }

        Ok(())
    }

    fn compute_long_running(
        &mut self,
        code_document: CodeDocument,
        _trigger: &CoreEngineTrigger,
        _execution_id: Option<Uuid>,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        let cancel_recv = self.reset_cancellation_channel();

        let (compute_long_running_send, mut compute_long_running_recv) =
            tokio::sync::mpsc::channel(1);

        tauri::async_runtime::spawn({
            let dismissed_suggestions = self.dismissed_suggestions.clone();
            let suggestions_arc = self.suggestions_arc.clone();

            async move {
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

                    if compute_long_running_send.is_closed() {
                        return;
                    }

                    _ = Self::compute_suggestions(
                        suggestions_arc,
                        dismissed_suggestions,
                        code_document,
                        compute_long_running_send.clone(),
                    )
                    .await;
                });

                tokio::select! {
                    _ = compute_long_running_recv.recv() => {
                        // Finished computing complexity refactoring suggestions
                    }
                    _ = cancel_recv => {
                        // Cancelled computing complexity refactoring suggestions
                    }
                }
            }
        });

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

    fn kind(&self) -> FeatureKind {
        FeatureKind::ComplexityRefactoring
    }
}

impl ComplexityRefactoring {
    pub fn new() -> Self {
        Self {
            suggestions_arc: Arc::new(Mutex::new(HashMap::new())),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
            dismissed_suggestions: Arc::new(Mutex::new(procedures::read_dismissed_suggestions())),
            cancel_long_running_task_send: None,
        }
    }

    fn reset_cancellation_channel(&mut self) -> oneshot::Receiver<&'static ()> {
        if let Some(sender) = self.cancel_long_running_task_send.take() {
            // Cancel previous task if it exists.
            _ = sender.send(&());
        }

        let (send, recv) = oneshot::channel();
        self.cancel_long_running_task_send = Some(send);
        recv
    }

    fn set_suggestions_to_recalculating(
        suggestions_arc: SuggestionsArcMutex,
        editor_window_uid: EditorWindowUid,
    ) -> Option<()> {
        suggestions_arc
            .lock()
            .get_mut(&editor_window_uid)?
            .values_mut()
            .filter(|s| s.state == SuggestionState::Ready)
            .for_each(|suggestion| suggestion.state = SuggestionState::Recalculating);

        Self::publish_to_frontend(suggestions_arc.lock().clone());

        Some(())
    }

    async fn compute_suggestions(
        suggestions_arc: SuggestionsArcMutex,
        dismissed_suggestions: Arc<Mutex<Vec<SuggestionHash>>>,
        code_document: CodeDocument,
        sender: tokio::sync::mpsc::Sender<()>,
    ) -> Result<(), FeatureError> {
        let window_uid = code_document.editor_window_props().window_uid;
        Self::set_suggestions_to_recalculating(suggestions_arc.clone(), window_uid);

        let old_suggestions = Self::get_suggestions_for_window(suggestions_arc.clone(), window_uid);

        let text_content = code_document
            .text_content()
            .as_ref()
            .ok_or(FeatureError::GenericError(
                ComplexityRefactoringError::InsufficientContext.into(),
            ))?
            .clone();

        let top_level_functions = SwiftFunction::get_top_level_functions(
            code_document
                .syntax_tree()
                .ok_or(FeatureError::GenericError(
                    ComplexityRefactoringError::InsufficientContext.into(),
                ))?,
            &text_content,
        )
        .map_err(|err| ComplexityRefactoringError::GenericError(err.into()))?;

        let file_path = code_document.file_path().clone();
        let mut s_exps = vec![];
        let mut suggestions: SuggestionsMap = HashMap::new();

        // We should spawn them all at once, and then wait for them to finish
        for function in top_level_functions {
            s_exps.push(function.props.node.to_sexp());
            suggestions.extend(Self::generate_suggestions_for_function(
                function,
                &text_content,
                &file_path,
                code_document
                    .syntax_tree()
                    .ok_or(FeatureError::GenericError(
                        ComplexityRefactoringError::InsufficientContext.into(),
                    ))?,
                suggestions_arc.clone(),
                dismissed_suggestions.clone(),
                code_document.editor_window_props().window_uid,
                sender.clone(),
            )?);
        }

        // Wait for all to finish, only then proceed
        suggestions_arc
            .lock()
            .insert(window_uid, suggestions.clone());

        let removed_suggestion_ids: Vec<Uuid> = old_suggestions
            .clone()
            .into_keys()
            .filter(|id| !suggestions.contains_key(id))
            .collect();

        remove_annotations_for_suggestions(removed_suggestion_ids.clone());

        Self::publish_to_frontend(suggestions_arc.lock().clone());

        Ok(())
    }

    pub fn publish_to_frontend(suggestions_per_window: SuggestionsPerWindow) {
        let mut fe_suggestions_per_window = HashMap::new();
        for (window_uid, suggestions) in suggestions_per_window {
            let fe_suggestions = suggestions
                .into_iter()
                .map(|(id, suggestion)| {
                    (
                        id,
                        map_refactoring_suggestion_to_fe_refactoring_suggestion(suggestion),
                    )
                })
                .collect::<HashMap<Uuid, FERefactoringSuggestion>>();
            fe_suggestions_per_window.insert(window_uid, fe_suggestions);
        }

        SuggestionEvent::ReplaceSuggestions(ReplaceSuggestionsMessage {
            suggestions: fe_suggestions_per_window,
        })
        .publish_to_tauri(&app_handle());
    }

    fn generate_suggestions_for_function(
        function: SwiftFunction,
        text_content: &XcodeText,
        file_path: &Option<String>,
        syntax_tree: &SwiftSyntaxTree,
        suggestions_arc: SuggestionsArcMutex,
        dismissed_suggestions_arc: Arc<Mutex<Vec<SuggestionHash>>>,
        window_uid: EditorWindowUid,
        sender: tokio::sync::mpsc::Sender<()>,
    ) -> Result<SuggestionsMap, ComplexityRefactoringError> {
        // This is heavy, should be done in parallel -> rayon, but since it takes TSTree which is not sent this is not trivial.
        let mut suggestions = Self::compute_suggestions_for_function(
            &function,
            suggestions_arc.clone(),
            &text_content,
            &syntax_tree,
            dismissed_suggestions_arc,
            window_uid,
        )?;

        // Compute annotations
        let mut suggestions_and_meta_infos: HashMap<
            Uuid,
            (
                RefactoringSuggestion,
                TextPosition,
                TextRange,
                Option<String>,
                Vec<String>,
            ),
        > = HashMap::new();

        for (id, suggestion) in suggestions.iter_mut() {
            let (suggestion_start_pos, suggestion_range, parent_node_kind, node_kinds) =
                Self::compute_complexity_annotations(
                    suggestion,
                    &function,
                    text_content,
                    id,
                    window_uid,
                )?;

            suggestions_and_meta_infos.insert(
                *id,
                (
                    suggestion.clone(),
                    suggestion_start_pos,
                    suggestion_range,
                    parent_node_kind,
                    node_kinds,
                ),
            );
        }

        for (
            id,
            (suggestion, suggestion_start_pos, suggestion_range, parent_node_kind, node_kinds),
        ) in suggestions_and_meta_infos.iter()
        {
            //
            // Spin up a task for each suggestion to run against SourceKit
            //
            tauri::async_runtime::spawn({
                let suggestion_start_pos = suggestion_start_pos.clone();
                let suggestion_range = suggestion_range.clone();
                let node_kinds = node_kinds.clone();
                let parent_node_kind = parent_node_kind.clone();

                let binded_text_content = text_content.clone();
                let binded_text_content_2 = text_content.clone();
                let binded_file_path = file_path.clone();
                let binded_file_path_2 = file_path.clone();
                let binded_suggestion = suggestion.clone();
                let binded_id: Uuid = *id;
                let binded_suggestions_arc = suggestions_arc.clone();
                let binded_suggestions_arc2 = suggestions_arc.clone();

                let sender = sender.clone();

                // For error reporting
                let serialized_slice = suggestion.serialized_slice.clone();

                async move {
                    // SourceKit -> very heavy
                    if sender.is_closed() {
                        return;
                    }

                    let edits = get_edits_for_method_extraction(
                        suggestion_start_pos,
                        suggestion_range.length,
                        &binded_text_content_2,
                        binded_file_path_2,
                    )
                    .await;

                    match edits {
                        Ok(edits) => {
                            if sender.is_closed() {
                                return;
                            }

                            _ = Self::update_suggestion_with_formatted_text_diff(
                                binded_id,
                                binded_suggestion,
                                edits,
                                binded_text_content,
                                binded_suggestions_arc,
                                binded_file_path,
                                window_uid,
                            )
                            .await;
                        }
                        Err(err) => {
                            //
                            let should_remove_suggestion = match err {
                                ComplexityRefactoringError::ExecutionCancelled(_) => false,
                                ComplexityRefactoringError::LspRejectedRefactoring(payload) => {
                                    warn!(
                                        ?payload,
                                        ?serialized_slice,
                                        ?node_kinds,
                                        ?parent_node_kind,
                                        "LSP rejected refactoring"
                                    );
                                    true
                                }
                                _ => {
                                    error!(?err, "Failed to perform refactoring");
                                    true
                                }
                            };

                            if should_remove_suggestion {
                                Self::remove_suggestion_and_publish(
                                    &window_uid,
                                    &binded_id,
                                    &binded_suggestions_arc2,
                                )
                                .unwrap_or_else(|e| {
                                    error!(?e, "Failed to remove suggestion when cleaning up after other error");
                                });
                            }
                        }
                    }
                }
            });
        }

        Ok(suggestions)
    }

    pub fn remove_suggestion_and_publish(
        window_uid: &EditorWindowUid,
        suggestion_id: &SuggestionId,
        suggestions_arc: &Arc<
            Mutex<HashMap<EditorWindowUid, HashMap<SuggestionId, RefactoringSuggestion>>>,
        >,
    ) -> Result<(), ComplexityRefactoringError> {
        let mut suggestions_per_window = suggestions_arc.lock();
        let suggestions = suggestions_per_window.get_mut(&window_uid).ok_or(
            ComplexityRefactoringError::SuggestionsForWindowNotFound(*window_uid),
        )?;

        remove_annotations_for_suggestions(vec![*suggestion_id]);
        suggestions.remove(&suggestion_id);
        Self::publish_to_frontend(suggestions_per_window.clone());
        Ok(())
    }

    fn compute_suggestions_for_function(
        function: &SwiftFunction,
        suggestions_arc: SuggestionsArcMutex,
        text_content: &XcodeText,
        syntax_tree: &SwiftSyntaxTree,
        dismissed_suggestions_arc: Arc<Mutex<Vec<SuggestionHash>>>,
        window_uid: EditorWindowUid,
    ) -> Result<SuggestionsMap, ComplexityRefactoringError> {
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

        let old_suggestions = Self::get_suggestions_for_window(suggestions_arc, window_uid);
        let old_suggestions_with_same_serialization: Vec<(&Uuid, &RefactoringSuggestion)> =
            old_suggestions
                .iter()
                .filter(|&(_, suggestion)| suggestion.serialized_slice == serialized_node_slice)
                .collect::<Vec<_>>();

        let id;
        let state;
        if old_suggestions_with_same_serialization.len() == 1 {
            // Re-identify ID with previous value to avoid unnecessary removal and addition
            id = *old_suggestions_with_same_serialization[0].0;
            state = match (*old_suggestions_with_same_serialization[0].1).state {
                SuggestionState::New => SuggestionState::New,
                SuggestionState::Ready | SuggestionState::Recalculating => {
                    SuggestionState::Recalculating
                }
            };
        } else {
            id = uuid::Uuid::new_v4();
            state = SuggestionState::New;
        };

        new_suggestions.insert(
            id,
            RefactoringSuggestion {
                state,
                serialized_slice: serialized_node_slice,
                main_function_name: function.get_name(),
                new_complexity,
                prev_complexity,
                old_text_content_string: None,
                new_text_content_string: None,
                start_index: None,
            },
        );

        Ok(new_suggestions)
    }

    fn update_suggestion(
        id: Uuid,
        updated_suggestion: &RefactoringSuggestion,
        suggestions_arc: SuggestionsArcMutex,
        window_uid: EditorWindowUid,
    ) {
        let mut suggestions_per_window = suggestions_arc.lock();
        if let Some(suggestions_map) = suggestions_per_window.get_mut(&window_uid) {
            if let Some(suggestion) = suggestions_map.get_mut(&id) {
                suggestion.clone_from(updated_suggestion);
                Self::publish_to_frontend(suggestions_per_window.clone());
            }
        }
    }

    async fn update_suggestion_with_formatted_text_diff(
        id: Uuid,
        mut suggestion: RefactoringSuggestion,
        edits: Vec<Edit>,
        text_content: XcodeText,
        suggestions_arc: SuggestionsArcMutex,
        file_path: Option<String>,
        window_uid: EditorWindowUid,
    ) {
        let (old_content, new_content) =
            Self::format_and_apply_edits_to_text_content(edits, text_content, file_path).await;

        suggestion.old_text_content_string = Some(old_content);
        suggestion.new_text_content_string = Some(new_content);
        suggestion.state = SuggestionState::Ready;
        Self::update_suggestion(id, &suggestion, suggestions_arc, window_uid);
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

        let formatted_new_content = match format_code(&edited_content, &file_path).await {
            Ok(content) => content,
            Err(e) => {
                error!(?e, "Failed to format during refactoring: new content");
                edited_content.as_string()
            }
        };

        let formatted_old_content = match format_code(&text_content, &file_path).await {
            Ok(content) => content,
            Err(e) => {
                error!(?e, "Failed to format during refactoring: old content");
                text_content.as_string()
            }
        };

        (formatted_old_content, formatted_new_content)
    }

    async fn perform_suggestion(
        code_document: CodeDocument,
        suggestion_id: SuggestionId,
        suggestions_arc: SuggestionsArcMutex,
    ) -> Result<(), ComplexityRefactoringError> {
        let window_uid = code_document.editor_window_props().window_uid;
        let suggestions = Self::get_suggestions_for_window(suggestions_arc.clone(), window_uid);

        let suggestion_to_apply = suggestions
            .get(&suggestion_id)
            .ok_or(ComplexityRefactoringError::SuggestionNotFound(
                suggestion_id.to_string(),
            ))?
            .clone();

        debug!(
            ?suggestion_to_apply,
            feature = FeatureKind::ComplexityRefactoring.to_string(),
            "Performing suggestion"
        );

        let text_range_to_scroll_to_after_performing =
            Self::get_text_position_to_scroll_to_after_performing(&suggestion_to_apply);

        let new_content = suggestion_to_apply.clone().new_text_content_string.ok_or(
            ComplexityRefactoringError::SuggestionIncomplete(suggestion_to_apply),
        )?;

        let text_content = code_document
            .text_content()
            .ok_or(ComplexityRefactoringError::InsufficientContext)?;

        match replace_text_content(
            &text_content,
            &XcodeText::from_str(&new_content),
            code_document.selected_text_range(),
        )
        .await
        {
            Ok(_) => {}
            Err(err) => {
                error!(?err, "Error replacing text content");
                return Err(ComplexityRefactoringError::GenericError(err.into()));
            }
        }

        Self::remove_suggestion_and_publish(&window_uid, &suggestion_id, &suggestions_arc)?;

        match text_range_to_scroll_to_after_performing {
            Ok(range) => {
                _ = set_selected_text_range(&range, &GetVia::Current);
            }
            Err(e) => {
                error!(
                    ?e,
                    "Error getting final cursor position after performing suggestion"
                );
                return Err(e);
            }
        }

        Ok(())
    }

    fn get_text_position_to_scroll_to_after_performing(
        suggestion: &RefactoringSuggestion,
    ) -> Result<TextRange, ComplexityRefactoringError> {
        let prev_text = suggestion.old_text_content_string.as_ref().ok_or(
            ComplexityRefactoringError::SuggestionIncomplete(suggestion.clone()),
        )?;
        let new_text: &String = suggestion.new_text_content_string.as_ref().ok_or(
            ComplexityRefactoringError::SuggestionIncomplete(suggestion.clone()),
        )?;

        let index = get_index_of_first_difference(prev_text, new_text)
            .ok_or(ComplexityRefactoringError::CouldNotGetCursorPositionAfterPerforming)?;
        Ok(TextRange { index, length: 0 })
    }

    fn determine_short_running_procedure(
        &self,
        trigger: &CoreEngineTrigger,
    ) -> Option<ComplexityRefactoringShortRunningProcedure> {
        match trigger {
            CoreEngineTrigger::OnUserCommand(UserCommand::PerformSuggestion(msg)) => {
                Some(ComplexityRefactoringShortRunningProcedure::PerformSuggestion(msg.id))
            }
            CoreEngineTrigger::OnUserCommand(UserCommand::DismissSuggestion(msg)) => {
                Some(ComplexityRefactoringShortRunningProcedure::DismissSuggestion(msg.id))
            }
            CoreEngineTrigger::OnUserCommand(UserCommand::SelectSuggestion(msg)) => msg
                .id
                .map(|id| ComplexityRefactoringShortRunningProcedure::SelectSuggestion(id)),
            _ => None,
        }
    }

    fn get_suggestions_for_window(
        suggestions_arc: SuggestionsArcMutex,
        window_uid: EditorWindowUid,
    ) -> SuggestionsMap {
        if let Some(suggestions) = suggestions_arc.lock().get_mut(&window_uid) {
            suggestions.clone()
        } else {
            HashMap::new()
        }
    }

    fn compute_complexity_annotations<'a>(
        suggestion: &'a mut RefactoringSuggestion,
        function: &'a SwiftFunction,
        text_content: &'a XcodeText,
        id: &'a Uuid,
        window_uid: usize,
    ) -> Result<(TextPosition, TextRange, Option<String>, Vec<String>), ComplexityRefactoringError>
    {
        let slice = NodeSlice::deserialize(&suggestion.serialized_slice, function.props.node)?;
        let suggestion_start_pos = TextPosition::from_TSPoint(&slice.nodes[0].start_position());
        let suggestion_end_pos =
            TextPosition::from_TSPoint(&slice.nodes.last().unwrap().end_position());
        let suggestion_range = TextRange::from_StartEndTextPosition(
            text_content,
            &suggestion_start_pos,
            &suggestion_end_pos,
        )
        .ok_or(ComplexityRefactoringError::GenericError(anyhow!(
            "Failed to derive suggestion range"
        )))?;
        let context_range = TextRange::from_StartEndTextPosition(
            &text_content,
            &function.get_first_char_position(),
            &function.get_last_char_position(),
        )
        .ok_or(ComplexityRefactoringError::GenericError(anyhow!(
            "Failed to derive context range"
        )))?;
        suggestion.start_index = Some(suggestion_range.index);
        set_annotation_group_for_extraction_and_context(
            *id,
            context_range,
            suggestion_range,
            window_uid,
        );

        let node_kinds = slice
            .nodes
            .iter()
            .map(|n| n.kind().to_string())
            .collect::<Vec<_>>();

        let parent_node_kind = slice
            .nodes
            .first()
            .and_then(|n| n.parent())
            .map(|n| n.kind().to_string());

        Ok((
            suggestion_start_pos,
            suggestion_range,
            parent_node_kind,
            node_kinds,
        ))
    }
}

fn map_refactoring_suggestion_to_fe_refactoring_suggestion(
    suggestion: RefactoringSuggestion,
) -> FERefactoringSuggestion {
    FERefactoringSuggestion {
        state: suggestion.state,
        new_text_content_string: suggestion.new_text_content_string,
        old_text_content_string: suggestion.old_text_content_string,
        new_complexity: suggestion.new_complexity,
        prev_complexity: suggestion.prev_complexity,
        main_function_name: suggestion.main_function_name,
        start_index: suggestion
            .start_index
            .expect("Suggestion start index should be set"),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ComplexityRefactoringError {
    #[error("Insufficient context for complexity refactoring")]
    InsufficientContext,
    #[error("Execution was cancelled: '{}'", 0)]
    ExecutionCancelled(Option<Uuid>),
    #[error("No suggestions found for window")]
    SuggestionsForWindowNotFound(usize),
    #[error("No suggestion found to apply")]
    SuggestionNotFound(String),
    #[error("Suggestion has incomplete state")]
    SuggestionIncomplete(RefactoringSuggestion),
    #[error("LSP rejected refactoring operation")]
    LspRejectedRefactoring(String),
    #[error("Failed to read or write dismissed suggestions file")]
    ReadWriteDismissedSuggestionsFailed,
    #[error("Could not derive final cursor position to scroll to after performing suggestion")]
    CouldNotGetCursorPositionAfterPerforming,
    #[error("Something went wrong when executing this ComplexityRefactoring feature.")]
    GenericError(#[source] anyhow::Error),
}
