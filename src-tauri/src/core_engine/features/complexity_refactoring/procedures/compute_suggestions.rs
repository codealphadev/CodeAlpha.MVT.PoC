use crate::{
    core_engine::{
        features::{
            complexity_refactoring::{
                check_for_method_extraction, create_annotation_group_for_extraction_and_context,
                method_extraction::{get_edits_for_method_extraction, MethodExtractionTask},
                remove_annotations_for_suggestions, ComplexityRefactoring,
                ComplexityRefactoringError, Edit, NodeSlice, RefactoringSuggestion, SuggestionHash,
                SuggestionState, SuggestionsArcMutex, SuggestionsMap,
            },
            FeatureSignals,
        },
        format_code,
        syntax_tree::{SwiftCodeBlockBase, SwiftFunction, SwiftSyntaxTree},
        CodeDocument, EditorWindowUid, TextPosition, TextRange, XcodeText,
    },
    utils::calculate_hash,
};
use anyhow::anyhow;
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tracing::debug;

use tokio::sync::mpsc;

use tracing::{error, warn};
use uuid::Uuid;

const MAX_ALLOWED_COMPLEXITY: isize = 9;

pub async fn compute_suggestions(
    suggestions_arc: SuggestionsArcMutex,
    dismissed_suggestions: Arc<Mutex<Vec<SuggestionHash>>>,
    code_document: CodeDocument,
    signals_sender: mpsc::Sender<FeatureSignals>,
) -> Result<(), ComplexityRefactoringError> {
    let window_uid = code_document.editor_window_props().window_uid;
    set_suggestions_to_recalculating(suggestions_arc.clone(), window_uid);

    let old_suggestions =
        ComplexityRefactoring::get_suggestions_for_window(suggestions_arc.clone(), window_uid);

    let text_content = code_document
        .text_content()
        .as_ref()
        .ok_or(ComplexityRefactoringError::InsufficientContext.into())?
        .clone();

    let top_level_functions = SwiftFunction::get_top_level_functions(
        code_document
            .syntax_tree()
            .ok_or(ComplexityRefactoringError::InsufficientContext.into())?,
        &text_content,
    )
    .map_err(|err| ComplexityRefactoringError::GenericError(err.into()))?;

    let file_path = code_document.file_path().clone();
    let mut s_exps = vec![];
    let mut suggestions: SuggestionsMap = HashMap::new();

    ComplexityRefactoring::verify_task_not_canceled(&signals_sender)?;

    // We should spawn them all at once, and then wait for them to finish
    for function in top_level_functions {
        s_exps.push(function.props.node.to_sexp());
        suggestions.extend(generate_suggestions_for_function(
            function,
            &text_content,
            &file_path,
            code_document
                .syntax_tree()
                .ok_or(ComplexityRefactoringError::InsufficientContext.into())?,
            suggestions_arc.clone(),
            dismissed_suggestions.clone(),
            code_document.editor_window_props().window_uid,
            signals_sender.clone(),
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

    ComplexityRefactoring::publish_to_frontend(suggestions_arc.lock().clone());

    Ok(())
}

fn generate_suggestions_for_function(
    function: SwiftFunction,
    text_content: &XcodeText,
    file_path: &Option<String>,
    syntax_tree: &SwiftSyntaxTree,
    suggestions_arc: SuggestionsArcMutex,
    dismissed_suggestions_arc: Arc<Mutex<Vec<SuggestionHash>>>,
    window_uid: EditorWindowUid,
    signals_sender: mpsc::Sender<FeatureSignals>,
) -> Result<SuggestionsMap, ComplexityRefactoringError> {
    // This is heavy, should be done in parallel -> rayon, but since it takes TSNodes which is not sent this is not trivial.
    let mut suggestions = compute_suggestions_for_function(
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
        let (suggestion_start_pos, suggestion_range, parent_node_kind, node_kinds, context_range) =
            compute_suggestion_metadata(suggestion, &function, text_content)?;

        create_annotation_group_for_extraction_and_context(
            *id,
            context_range,
            suggestion_range,
            window_uid,
        );

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

    for (id, (suggestion, suggestion_start_pos, suggestion_range, parent_node_kind, node_kinds)) in
        suggestions_and_meta_infos.iter()
    {
        //
        // Spin up a task for each suggestion to run against SourceKit
        //
        tauri::async_runtime::spawn({
            let method_extraction_task = MethodExtractionTask {
                text_content: text_content.clone(),
                start_position: suggestion_start_pos.clone(),
                range_length: suggestion_range.length,
                file_path: file_path.clone(),
            };

            let node_kinds = node_kinds.clone();
            let parent_node_kind = parent_node_kind.clone();

            let text_content = text_content.clone();
            let file_path = file_path.clone();

            let suggestion = suggestion.clone();
            let id: Uuid = *id;
            let suggestions_arc = suggestions_arc.clone();

            let signals_sender = signals_sender.clone();

            // For error reporting
            let serialized_slice = suggestion.serialized_slice.clone();

            async move {
                // SourceKit -> very heavy
                if ComplexityRefactoring::verify_task_not_canceled(&signals_sender).is_err() {
                    return;
                };

                let edits =
                    get_edits_for_method_extraction(method_extraction_task, &signals_sender).await;

                match edits {
                    Ok(edits) => {
                        if ComplexityRefactoring::verify_task_not_canceled(&signals_sender).is_err()
                        {
                            return;
                        };

                        _ = update_suggestion_with_formatted_text_diff(
                            id,
                            suggestion,
                            edits,
                            text_content,
                            suggestions_arc,
                            file_path,
                            window_uid,
                            &signals_sender,
                        )
                        .await;

                        _ = signals_sender
                            .send(FeatureSignals::ComputationCompleted)
                            .await;
                    }
                    Err(err) => {
                        let should_remove_suggestion = match err {
                            ComplexityRefactoringError::ExecutionCancelled => false,
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
                            _ = ComplexityRefactoring::remove_suggestion_and_publish(
                            &window_uid,
                            &id,
                            &suggestions_arc,
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

async fn update_suggestion_with_formatted_text_diff(
    id: Uuid,
    mut suggestion: RefactoringSuggestion,
    edits: Vec<Edit>,
    text_content: XcodeText,
    suggestions_arc: SuggestionsArcMutex,
    file_path: Option<String>,
    window_uid: EditorWindowUid,
    signals_sender: &mpsc::Sender<FeatureSignals>,
) {
    if ComplexityRefactoring::verify_task_not_canceled(&signals_sender).is_err() {
        return;
    };

    let (old_content, new_content) =
        format_and_apply_edits_to_text_content(edits, text_content, file_path).await;

    suggestion.old_text_content_string = Some(old_content);
    suggestion.new_text_content_string = Some(new_content);
    suggestion.state = SuggestionState::Ready;
    ComplexityRefactoring::update_suggestion(id, &suggestion, suggestions_arc, window_uid);
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

    let old_suggestions =
        ComplexityRefactoring::get_suggestions_for_window(suggestions_arc, window_uid);
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

fn compute_suggestion_metadata<'a>(
    suggestion: &'a mut RefactoringSuggestion,
    function: &'a SwiftFunction,
    text_content: &'a XcodeText,
) -> Result<
    (
        TextPosition,
        TextRange,
        Option<String>,
        Vec<String>,
        TextRange,
    ),
    ComplexityRefactoringError,
> {
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
        context_range,
    ))
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

    ComplexityRefactoring::publish_to_frontend(suggestions_arc.lock().clone());

    Some(())
}
