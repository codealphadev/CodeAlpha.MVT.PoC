use super::{create_annotation_group_for_extraction_and_context, NodeSlice, SerializedNodeSlice};
use crate::{
    app_handle,
    core_engine::{
        annotations_manager::{AnnotationKind, GetAnnotationInGroupVia},
        events::{models::ReplaceSuggestionsMessage, AnnotationManagerEvent, SuggestionEvent},
        features::{
            complexity_refactoring::{
                check_for_method_extraction, method_extraction::get_edits_for_method_extraction,
                remove_annotations_for_suggestions,
            },
            feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
            UserCommand,
        },
        format_code,
        syntax_tree::{SwiftCodeBlockBase, SwiftFunction, SwiftSyntaxTree},
        CodeDocument, EditorWindowUid, TextPosition, TextRange, XcodeText,
    },
    platform::macos::{
        replace_text_content, xcode::actions::replace_range_with_clipboard_text, GetVia,
    },
    utils::calculate_hash,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};
use anyhow::anyhow;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, sync::Arc};
use tracing::debug;
use tracing::error;
use tracing::warn;
use ts_rs::TS;
use uuid::Uuid;

type SuggestionHash = u64;
pub type SuggestionId = uuid::Uuid;
type SuggestionsMap = HashMap<SuggestionId, RefactoringSuggestion>;
type SuggestionsPerWindow = HashMap<EditorWindowUid, SuggestionsMap>;
type SuggestionsArcMutex = Arc<Mutex<SuggestionsPerWindow>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    pub text: XcodeText,
    pub start_index: usize,
    pub end_index: usize,
}

enum ComplexityRefactoringProcedure {
    ComputeSuggestions,
    PerformOperation(SuggestionId),
    DismissSuggestion(SuggestionId),
    SelectSuggestion(SuggestionId),
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub struct FERefactoringSuggestion {
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
    pub edits: Vec<Edit>,
    pub new_complexity: isize,
    pub prev_complexity: isize,
    pub main_function_name: Option<String>,
    pub serialized_slice: SerializedNodeSlice,
}

pub fn map_refactoring_suggestion_to_fe_refactoring_suggestion(
    suggestion: RefactoringSuggestion,
) -> FERefactoringSuggestion {
    FERefactoringSuggestion {
        new_text_content_string: suggestion.new_text_content_string,
        old_text_content_string: suggestion.old_text_content_string,
        new_complexity: suggestion.new_complexity,
        prev_complexity: suggestion.prev_complexity,
        main_function_name: suggestion.main_function_name,
    }
}

pub struct ComplexityRefactoring {
    is_activated: bool,
    suggestions_arc: SuggestionsArcMutex,
    dismissed_suggestions: Arc<Mutex<Vec<SuggestionHash>>>,
}

const MAX_ALLOWED_COMPLEXITY: isize = 9;

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
                        self.suggestions_arc.lock().clear();
                        e
                    })
                }
                ComplexityRefactoringProcedure::PerformOperation(id) => self
                    .perform_operation(code_document, id)
                    .map_err(|e| e.into()),
                ComplexityRefactoringProcedure::DismissSuggestion(id) => self
                    .dismiss_suggestion(code_document, id)
                    .map_err(|e| e.into()),
                ComplexityRefactoringProcedure::SelectSuggestion(id) => {
                    self.select_suggestion(id).map_err(|e| e.into())
                }
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
    pub fn new() -> Self {
        Self {
            suggestions_arc: Arc::new(Mutex::new(HashMap::new())),
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
            dismissed_suggestions: Arc::new(Mutex::new(read_dismissed_suggestions())),
        }
    }

    fn compute_suggestions(&mut self, code_document: &CodeDocument) -> Result<(), FeatureError> {
        debug!("Computing suggestions for complexity refactoring");
        let window_uid = code_document.editor_window_props().window_uid;
        let suggestions_arc = self.suggestions_arc.clone();
        let old_suggestions = Self::get_suggestions_for_window(suggestions_arc.clone(), window_uid);

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
        let mut suggestions: SuggestionsMap = HashMap::new();

        for function in top_level_functions {
            s_exps.push(function.props.node.to_sexp());
            suggestions.extend(Self::generate_suggestions_for_function(
                function,
                &text_content,
                &file_path,
                code_document.syntax_tree(),
                suggestions_arc.clone(),
                self.dismissed_suggestions.clone(),
                code_document.editor_window_props().window_uid,
            )?);
        }
        self.suggestions_arc
            .lock()
            .insert(window_uid, suggestions.clone());

        let added_suggestions_count = suggestions
            .clone()
            .iter()
            .filter(|(id, _)| !old_suggestions.contains_key(&id))
            .count();

        let removed_suggestion_ids: Vec<Uuid> = old_suggestions
            .clone()
            .into_keys()
            .filter(|id| !suggestions.contains_key(id))
            .collect();

        remove_annotations_for_suggestions(removed_suggestion_ids.clone());

        if removed_suggestion_ids.len() > 0 || added_suggestions_count > 0 {
            Self::publish_to_frontend(self.suggestions_arc.lock().clone());
        }

        Ok(())
    }

    fn publish_to_frontend(suggestions_per_window: SuggestionsPerWindow) {
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
    ) -> Result<SuggestionsMap, ComplexityRefactoringError> {
        let suggestions = Self::compute_suggestions_for_function(
            &function,
            suggestions_arc.clone(),
            &text_content,
            &syntax_tree,
            dismissed_suggestions_arc,
            window_uid,
        )?;

        for (id, suggestion) in suggestions.iter() {
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

            create_annotation_group_for_extraction_and_context(
                *id,
                context_range,
                suggestion_range,
                window_uid,
            );

            let binded_text_content = text_content.clone();
            let binded_text_content_2 = text_content.clone();
            let binded_file_path = file_path.clone();
            let binded_file_path_2 = file_path.clone();
            let binded_suggestion = suggestion.clone();
            let binded_id: Uuid = *id;
            let binded_old_suggestions = suggestions_arc.clone();

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
                    _ = get_edits_for_method_extraction(
                        suggestion_start_pos,
                        suggestion_range.length,
                        move |edits: Vec<Edit>| {
                            Self::update_suggestion_with_formatted_text_diff(
                                binded_id,
                                binded_suggestion,
                                edits,
                                binded_text_content,
                                binded_old_suggestions,
                                binded_file_path,
                                window_uid,
                            )
                        },
                        &binded_text_content_2,
                        binded_file_path_2,
                    )
                    .await
                    .map_err(|e| match e {
                        ComplexityRefactoringError::LspRejectedRefactoring(payload) => {
                            warn!(
                                ?payload,
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
                edits: Vec::new(),
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

    fn update_suggestion_with_formatted_text_diff(
        id: Uuid,
        mut suggestion: RefactoringSuggestion,
        edits: Vec<Edit>,
        text_content: XcodeText,
        suggestions_arc: SuggestionsArcMutex,
        file_path: Option<String>,
        window_uid: EditorWindowUid,
    ) {
        tauri::async_runtime::spawn(async move {
            let (old_content, new_content) = Self::format_and_apply_edits_to_text_content(
                edits.clone(),
                text_content,
                file_path,
            )
            .await;

            suggestion.old_text_content_string = Some(old_content);
            suggestion.new_text_content_string = Some(new_content);
            suggestion.edits = edits;
            Self::update_suggestion(id, &suggestion, suggestions_arc, window_uid);
        });
    }

    async fn format_and_apply_edits_to_text_content(
        mut edits: Vec<Edit>,
        text_content: XcodeText,
        _file_path: Option<String>,
    ) -> (String, String) {
        let mut edited_content = text_content.clone();

        edits.sort_by_key(|e| e.start_index);
        edits.reverse();

        for edit in edits {
            edited_content.replace_range(edit.start_index..edit.end_index, edit.text);
        }

        // let formatted_new_content = match format_code(&edited_content.as_string(), &file_path).await
        // {
        //     Ok(content) => content,
        //     Err(e) => {
        //         error!(?e, "Failed to format during refactoring: new content");
        //         edited_content.as_string()
        //     }
        // };

        // let formatted_old_content = match format_code(&text_content.as_string(), &file_path).await {
        //     Ok(content) => content,
        //     Err(e) => {
        //         error!(?e, "Failed to format during refactoring: old content");
        //         text_content.as_string()
        //     }
        // };

        (text_content.as_string(), edited_content.as_string())
    }

    fn perform_operation(
        &mut self,
        code_document: &CodeDocument,
        suggestion_id: SuggestionId,
    ) -> Result<(), ComplexityRefactoringError> {
        println!("perform_operation");

        let suggestions = Self::get_suggestions_for_window(
            self.suggestions_arc.clone(),
            code_document.editor_window_props().window_uid,
        );

        let suggestion_to_apply = suggestions
            .get(&suggestion_id)
            .ok_or(ComplexityRefactoringError::SuggestionNotFound(
                suggestion_id.to_string(),
            ))?
            .clone();

        let new_content = suggestion_to_apply.clone().new_text_content_string.ok_or(
            ComplexityRefactoringError::SuggestionIncomplete(suggestion_to_apply.clone()),
        )?;

        let text_content = code_document
            .text_content()
            .as_ref()
            .ok_or(ComplexityRefactoringError::InsufficientContext)?;

        println!("COME HERE!");

        tauri::async_runtime::spawn({
            let selected_text_range = code_document.selected_text_range().clone();
            let text_content = text_content.clone();
            let suggestions_arc = self.suggestions_arc.clone();
            let editor_window_uid = code_document.editor_window_props().window_uid;

            let mut suggestion_to_apply = suggestion_to_apply.clone();

            async move {
                suggestion_to_apply.edits.sort_by_key(|e| e.start_index);
                suggestion_to_apply.edits.reverse();

                for edit in suggestion_to_apply.edits {
                    replace_range_with_clipboard_text(
                        &app_handle(),
                        &GetVia::Current,
                        &TextRange {
                            index: edit.start_index,
                            length: edit.end_index - edit.start_index,
                        },
                        Some(&edit.text.as_string()),
                        true,
                    )
                    .await;
                    println!("Replaced range with clipboard text");
                }

                let mut suggestions_per_window = suggestions_arc.lock();
                if let Some(suggestions) = suggestions_per_window.get_mut(&editor_window_uid) {
                    suggestions.remove(&suggestion_id);

                    remove_annotations_for_suggestions(vec![suggestion_id]);
                    Self::publish_to_frontend(suggestions_per_window.clone());
                }
            }
        });

        Ok(())
    }

    fn select_suggestion(
        &mut self,
        suggestion_id: SuggestionId,
    ) -> Result<(), ComplexityRefactoringError> {
        AnnotationManagerEvent::ScrollToAnnotationInGroup((
            suggestion_id,
            GetAnnotationInGroupVia::Kind(AnnotationKind::ExtractionStartChar),
        ))
        .publish_to_tauri();

        Ok(())
    }

    fn dismiss_suggestion(
        &mut self,
        code_document: &CodeDocument,
        suggestion_id: SuggestionId,
    ) -> Result<(), ComplexityRefactoringError> {
        let window_uid = code_document.editor_window_props().window_uid;
        let mut suggestions_per_window = self.suggestions_arc.lock();
        let suggestions = suggestions_per_window.get_mut(&window_uid).ok_or(
            ComplexityRefactoringError::SuggestionsForWindowNotFound(window_uid),
        )?;
        let suggestion_to_dismiss = suggestions.get(&suggestion_id).ok_or(
            ComplexityRefactoringError::SuggestionNotFound(suggestion_id.to_string()),
        )?;

        let hash = write_dismissed_suggestion(&suggestion_to_dismiss.serialized_slice)?;
        self.dismissed_suggestions.lock().push(hash);

        suggestions.remove(&suggestion_id);
        remove_annotations_for_suggestions(vec![suggestion_id]);

        Self::publish_to_frontend(suggestions_per_window.clone());
        Ok(())
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
            CoreEngineTrigger::OnUserCommand(UserCommand::DismissSuggestion(msg)) => {
                Some(ComplexityRefactoringProcedure::DismissSuggestion(msg.id))
            }
            CoreEngineTrigger::OnUserCommand(UserCommand::SelectSuggestion(msg)) => {
                Some(ComplexityRefactoringProcedure::SelectSuggestion(msg.id))
            }
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
}

const DISMISSED_SUGGESTIONS_FILE_NAME: &str = "dismissed_suggestions.json";

fn write_dismissed_suggestion(
    suggestion: &SerializedNodeSlice,
) -> Result<SuggestionHash, ComplexityRefactoringError> {
    let hash = calculate_hash::<SerializedNodeSlice>(&suggestion);
    let app_dir = app_handle()
        .path_resolver()
        .app_dir()
        .ok_or(ComplexityRefactoringError::ReadWriteDismissedSuggestionsFailed)?;
    let path = app_dir.join(DISMISSED_SUGGESTIONS_FILE_NAME);
    let mut suggestions: Vec<SuggestionHash> = vec![];
    if path.exists() {
        if let Ok(file) = fs::read_to_string(&path) {
            suggestions = serde_json::from_str(&file).unwrap();
        }
    }

    if suggestions.contains(&hash) {
        return Ok(hash);
    }

    suggestions.push(hash);
    let suggestions_string = serde_json::to_string(&suggestions)
        .map_err(|_| ComplexityRefactoringError::ReadWriteDismissedSuggestionsFailed)?;
    fs::create_dir_all(app_dir)
        .map_err(|_| ComplexityRefactoringError::ReadWriteDismissedSuggestionsFailed)?;
    fs::write(&path, suggestions_string)
        .map_err(|_| ComplexityRefactoringError::ReadWriteDismissedSuggestionsFailed)?;

    Ok(hash)
}

fn read_dismissed_suggestions() -> Vec<SuggestionHash> {
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

#[derive(thiserror::Error, Debug)]
pub enum ComplexityRefactoringError {
    #[error("Insufficient context for complexity refactoring")]
    InsufficientContext,
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
    #[error("Something went wrong when executing this ComplexityRefactoring feature.")]
    GenericError(#[source] anyhow::Error),
}
