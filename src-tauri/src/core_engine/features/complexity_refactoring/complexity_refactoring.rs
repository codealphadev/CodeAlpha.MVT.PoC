use super::{
    procedures::{self, perform_suggestion},
    FERefactoringSuggestion, RefactoringSuggestion, SuggestionHash, SuggestionId,
    SuggestionsArcMutex, SuggestionsPerWindow,
};
use crate::{
    app_handle,
    core_engine::{
        events::{models::ReplaceSuggestionsMessage, SuggestionEvent},
        features::{
            complexity_refactoring::{remove_annotations_for_suggestions, SuggestionsMap},
            feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
            FeatureKind, FeatureSignals, UserCommand,
        },
        get_index_of_first_difference, CodeDocument, EditorWindowUid, TextRange,
    },
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};
use lazy_static::lazy_static;

use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tauri::api::process::CommandChild;

use tokio::sync::mpsc;

use tracing::error;
use uuid::Uuid;

lazy_static! {
    pub static ref CURRENT_COMPLEXITY_REFACTORING_EXECUTION_ID: Mutex<Option<Uuid>> =
        Mutex::new(None);
}

enum ComplexityRefactoringProcedure {
    PerformSuggestion(SuggestionId),
    DismissSuggestion(SuggestionId),
    SelectSuggestion(SuggestionId),
    ComputeSuggestions,
}

pub struct ComplexityRefactoring {
    is_activated: bool,
    suggestions_arc: SuggestionsArcMutex,
    dismissed_suggestions_arc: Arc<Mutex<Vec<SuggestionHash>>>,

    cancel_long_running_task_send: Option<mpsc::Sender<&'static ()>>,
}

impl FeatureBase for ComplexityRefactoring {
    fn compute(
        &mut self,
        code_document: CodeDocument,
        trigger: CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        let cancelation_event_recv = self.cancel_complexity_refactoring_task();

        tauri::async_runtime::spawn({
            let dismissed_suggestions_arc = self.dismissed_suggestions_arc.clone();
            let suggestions_arc = self.suggestions_arc.clone();

            async move {
                if let Some(procedure) = Self::determine_procedure(&trigger) {
                    if let Err(e) = match procedure {
                        ComplexityRefactoringProcedure::PerformSuggestion(id) => {
                            perform_suggestion(code_document, id, suggestions_arc).await
                        }
                        ComplexityRefactoringProcedure::DismissSuggestion(id) => {
                            procedures::dismiss_suggestion(
                                code_document,
                                id,
                                suggestions_arc,
                                dismissed_suggestions_arc,
                            )
                            .await
                        }
                        ComplexityRefactoringProcedure::SelectSuggestion(id) => {
                            procedures::select_suggestion(id).await
                        }
                        ComplexityRefactoringProcedure::ComputeSuggestions => {
                            // See Tokio Select Cancellation pattern -> https://tokio.rs/tokio/tutorial/select, chapter Canceling
                            let (feature_signals_send, feature_signals_recv) =
                                tokio::sync::mpsc::channel(1);

                            tauri::async_runtime::spawn(async move {
                                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                                if feature_signals_send.is_closed() {
                                    return;
                                }

                                match procedures::compute_suggestions(
                                    suggestions_arc,
                                    dismissed_suggestions_arc,
                                    code_document,
                                    feature_signals_send.clone(),
                                )
                                .await
                                {
                                    Err(ComplexityRefactoringError::ExecutionCancelled) => (),
                                    Err(e) => error!(?e, "Error while computing suggestions"),
                                    Ok(_) => (),
                                }
                            });

                            Self::handle_signals(feature_signals_recv, cancelation_event_recv)
                                .await;

                            Ok(())
                        }
                    } {
                        error!(?e, "Error while performing procedure");
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

    fn should_compute(_kind: &FeatureKind, trigger: &CoreEngineTrigger) -> bool {
        Self::determine_procedure(trigger).is_some()
    }

    fn requires_ai(_kind: &FeatureKind, _trigger: &CoreEngineTrigger) -> bool {
        false
    }
}

impl ComplexityRefactoring {
    pub fn new() -> Self {
        Self {
            suggestions_arc: Arc::new(Mutex::new(HashMap::new())),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
            dismissed_suggestions_arc: Arc::new(Mutex::new(
                procedures::read_dismissed_suggestions(),
            )),
            cancel_long_running_task_send: None,
        }
    }

    fn determine_procedure(trigger: &CoreEngineTrigger) -> Option<ComplexityRefactoringProcedure> {
        match trigger {
            CoreEngineTrigger::OnUserCommand(UserCommand::PerformSuggestion(msg)) => {
                Some(ComplexityRefactoringProcedure::PerformSuggestion(msg.id))
            }
            CoreEngineTrigger::OnUserCommand(UserCommand::DismissSuggestion(msg)) => {
                Some(ComplexityRefactoringProcedure::DismissSuggestion(msg.id))
            }
            CoreEngineTrigger::OnUserCommand(UserCommand::SelectSuggestion(msg)) => msg
                .id
                .map(|id| ComplexityRefactoringProcedure::SelectSuggestion(id)),
            CoreEngineTrigger::OnTextContentChange => {
                Some(ComplexityRefactoringProcedure::ComputeSuggestions)
            }
            _ => None,
        }
    }

    fn cancel_complexity_refactoring_task(&mut self) -> mpsc::Receiver<&'static ()> {
        if let Some(sender) = self.cancel_long_running_task_send.take() {
            // Cancel previous task if it exists.
            _ = sender.send(&());
        }

        let (send, recv) = mpsc::channel(1);
        self.cancel_long_running_task_send = Some(send);
        recv
    }

    pub fn verify_task_not_canceled(
        signals_sender: &mpsc::Sender<FeatureSignals>,
    ) -> Result<(), ComplexityRefactoringError> {
        if signals_sender.is_closed() {
            return Err(ComplexityRefactoringError::ExecutionCancelled);
        } else {
            Ok(())
        }
    }

    async fn handle_signals(
        mut feature_signals_recv: mpsc::Receiver<FeatureSignals>,
        mut cancelation_event_recv: mpsc::Receiver<&()>,
    ) {
        let mut swift_lsp_commands: Vec<CommandChild> = vec![];
        loop {
            tokio::select! {
                signal = feature_signals_recv.recv() => {
                    if let Some(signal) = signal {
                        match signal {
                            FeatureSignals::ComputationCompleted => {
                                break;
                            }
                            FeatureSignals::SwiftLspCommandSpawned(command) => {
                                swift_lsp_commands.push(command);
                            }
                        }
                    }

                }
                _ = cancelation_event_recv.recv() => {
                    for command in swift_lsp_commands {
                        _ = command.kill();
                    }
                    break;
                }
            }
        }
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

    pub fn update_suggestion(
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

    pub fn get_text_position_to_scroll_to_after_performing(
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

    pub fn get_suggestions_for_window(
        suggestions_arc: SuggestionsArcMutex,
        window_uid: EditorWindowUid,
    ) -> SuggestionsMap {
        if let Some(suggestions) = suggestions_arc.lock().get_mut(&window_uid) {
            suggestions.clone()
        } else {
            HashMap::new()
        }
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
    #[error("Execution was cancelled")]
    ExecutionCancelled,
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
