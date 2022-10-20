use std::{fs, sync::Arc};
use tracing::{debug, error};

use parking_lot::Mutex;

use crate::{
    app_handle,
    core_engine::{
        features::{complexity_refactoring::{
            remove_annotations_for_suggestions, ComplexityRefactoringError, SerializedNodeSlice,
            SuggestionHash, SuggestionId, SuggestionsArcMutex,
        }, ComplexityRefactoring},
        CodeDocument, EditorWindowUid,
    },
    utils::calculate_hash,
};

const DISMISSED_SUGGESTIONS_FILE_NAME: &str = "dismissed_suggestions.json";

pub async fn dismiss_suggestion(
    code_document: CodeDocument,
    suggestion_id: SuggestionId,
    suggestions_arc: SuggestionsArcMutex,
    dismissed_suggestions: Arc<Mutex<Vec<SuggestionHash>>>,
) -> Result<(), ComplexityRefactoringError> {
    let window_uid = code_document.editor_window_props().window_uid;

    let hash =
        write_dismissed_suggestion(suggestion_id, window_uid, suggestions_arc.clone()).await?;
    dismissed_suggestions.lock().push(hash);

    let mut suggestions_per_window = suggestions_arc.lock();
    let suggestions = suggestions_per_window.get_mut(&window_uid).ok_or(
        ComplexityRefactoringError::SuggestionsForWindowNotFound(window_uid),
    )?;
    suggestions.remove(&suggestion_id);
    remove_annotations_for_suggestions(vec![suggestion_id]);

    ComplexityRefactoring::publish_to_frontend(suggestions_per_window.clone());
    Ok(())
}

async fn write_dismissed_suggestion(
    suggestion_id: SuggestionId,
    window_uid: EditorWindowUid,
    suggestions_arc: SuggestionsArcMutex,
) -> Result<SuggestionHash, ComplexityRefactoringError> {
    let mut suggestions_per_window;
    {
        suggestions_per_window = suggestions_arc.lock().clone();
    }

    let suggestions_for_window_uid = suggestions_per_window.get_mut(&window_uid).ok_or(
        ComplexityRefactoringError::SuggestionsForWindowNotFound(window_uid),
    )?;
    let suggestion_to_dismiss = suggestions_for_window_uid.get(&suggestion_id).ok_or(
        ComplexityRefactoringError::SuggestionNotFound(suggestion_id.to_string()),
    )?;

    let hash = calculate_hash::<SerializedNodeSlice>(&suggestion_to_dismiss.serialized_slice);
    let app_dir = app_handle()
        .path_resolver()
        .app_dir()
        .ok_or(ComplexityRefactoringError::ReadWriteDismissedSuggestionsFailed)?;
    let path = app_dir.join(DISMISSED_SUGGESTIONS_FILE_NAME);
    let mut suggestions: Vec<SuggestionHash> = vec![];
    if path.exists() {
        if let Ok(file) = tokio::fs::read_to_string(&path).await {
            suggestions = serde_json::from_str(&file).unwrap();
        }
    }

    if suggestions.contains(&hash) {
        return Ok(hash);
    }

    suggestions.push(hash);
    let suggestions_string = serde_json::to_string(&suggestions)
        .map_err(|_| ComplexityRefactoringError::ReadWriteDismissedSuggestionsFailed)?;
    tokio::fs::create_dir_all(app_dir)
        .await
        .map_err(|_| ComplexityRefactoringError::ReadWriteDismissedSuggestionsFailed)?;
    tokio::fs::write(&path, suggestions_string)
        .await
        .map_err(|_| ComplexityRefactoringError::ReadWriteDismissedSuggestionsFailed)?;

    Ok(hash)
}

pub fn read_dismissed_suggestions() -> Vec<SuggestionHash> {
    if let Some(app_dir) = app_handle().path_resolver().app_dir() {
        let path = app_dir.join(DISMISSED_SUGGESTIONS_FILE_NAME);
        if let Ok(file) = fs::read_to_string(&path) {
            if let Ok(suggestions) = serde_json::from_str::<Vec<SuggestionHash>>(&file) {
                debug!(?suggestions, ?path, "Read dismissed suggestions file");
                return suggestions;
            } else {
                error!(DISMISSED_SUGGESTIONS_FILE_NAME, "Error parsing file");
            }
        } else {
            debug!(?path, "No dismissed suggestions file found");
        }
    } else {
        error!(DISMISSED_SUGGESTIONS_FILE_NAME, "Error getting app dir");
    }
    vec![]
}
