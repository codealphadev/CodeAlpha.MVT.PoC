use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use strum::IntoEnumIterator;
use tauri::Manager;
use tokio::sync::oneshot::{self, Receiver};
use tracing::{debug, error};

use crate::{
    app_handle,
    app_state::CoreEngineStateCache,
    platform::macos::{get_textarea_content, get_textarea_file_path, GetVia, XcodeError},
};

use super::{
    annotations_manager::{AnnotationsManager, AnnotationsManagerTrait},
    features::{
        hash_trigger_and_feature, BracketHighlight, ComplexityRefactoring, CoreEngineTrigger,
        DocsGenerator, Feature, FeatureBase, FeatureError, FeatureKind, FeatureProcedure,
        SwiftFormatter,
    },
    listeners::{user_interaction::user_interaction_listener, xcode::xcode_listener},
    log_list_of_module_names,
    syntax_tree::SwiftSyntaxTree,
    CodeDocument, EditorWindowProps, TextRange, XcodeText,
};

pub type EditorWindowUid = usize;

#[derive(thiserror::Error, Debug)]
pub enum CoreEngineError {
    #[error("There exists no CodeDocument with window_uid {0}.")]
    CodeDocNotFound(EditorWindowUid),
    #[error("Context missing to proceed: {0}.")]
    MissingContext(String),
    #[error("Something went wrong.")]
    GenericError(#[source] anyhow::Error),
}

impl From<FeatureError> for CoreEngineError {
    fn from(cause: FeatureError) -> Self {
        CoreEngineError::GenericError(cause.into())
    }
}

impl From<XcodeError> for CoreEngineError {
    fn from(cause: XcodeError) -> Self {
        CoreEngineError::GenericError(cause.into())
    }
}

#[derive(Debug, Clone)]
enum CodeDocUpdate {
    Finished,
    Aborded,
}

type FeatureProcedureSchedule = HashMap<
    u64,
    (
        FeatureKind,
        CoreEngineTrigger,
        FeatureProcedure,
        EditorWindowUid,
    ),
>;

pub struct CoreEngine {
    pub app_handle: tauri::AppHandle,

    /// List of open code documents.
    code_documents: Arc<Mutex<HashMap<EditorWindowUid, CodeDocument>>>,

    /// Features include bracket highlighting, docs generation and formatters.
    features: Arc<Mutex<HashMap<FeatureKind, Arc<Mutex<Feature>>>>>,

    ai_features_active: bool,
    swift_format_on_cmd_s_active: bool,

    /// Annotations manager handles where to draw annotations on the code editor via the CodeOverlay window
    _annotations_manager: Arc<Mutex<AnnotationsManager>>,

    /// We only allow a the most recent combination of trigger and feature to be scheduled for execution.
    /// Any newly scheduled feature execution replaces the previous one from the hash map.
    feature_procedures_schedule: Arc<Mutex<FeatureProcedureSchedule>>,

    cancel_code_doc_update_task_send: Option<oneshot::Sender<&'static ()>>,
    finished_code_doc_update_task_recv: Option<Receiver<&'static CodeDocUpdate>>,
    awaiting_code_doc_update_task: Arc<Mutex<()>>,
}

impl CoreEngine {
    pub fn new() -> Self {
        let annotations_manager = Arc::new(Mutex::new(AnnotationsManager::new()));
        AnnotationsManager::start_event_listeners(&annotations_manager);

        let ai_features_active =
            if let Some(cache) = app_handle().try_state::<CoreEngineStateCache>() {
                cache.0.lock().ai_features_active
            } else {
                true
            };

        let swift_format_on_cmd_s_active =
            if let Some(cache) = app_handle().try_state::<CoreEngineStateCache>() {
                cache.0.lock().swift_format_on_cmd_s
            } else {
                true
            };

        let mut features = HashMap::new();
        features.insert(
            FeatureKind::BracketHighlight,
            Arc::new(Mutex::new(Feature::BracketHighlighting(
                BracketHighlight::new(),
            ))),
        );
        features.insert(
            FeatureKind::DocsGeneration,
            Arc::new(Mutex::new(Feature::DocsGeneration(DocsGenerator::new()))),
        );
        features.insert(
            FeatureKind::Formatter,
            Arc::new(Mutex::new(Feature::Formatter(SwiftFormatter::new()))),
        );
        features.insert(
            FeatureKind::ComplexityRefactoring,
            Arc::new(Mutex::new(Feature::ComplexityRefactoring(
                ComplexityRefactoring::new(),
            ))),
        );

        let (send, _) = oneshot::channel();

        Self {
            app_handle: app_handle(),
            code_documents: Arc::new(Mutex::new(HashMap::new())),
            ai_features_active,
            features: Arc::new(Mutex::new(features)),
            _annotations_manager: annotations_manager,
            feature_procedures_schedule: Arc::new(Mutex::new(HashMap::new())),
            cancel_code_doc_update_task_send: Some(send),
            finished_code_doc_update_task_recv: None,
            awaiting_code_doc_update_task: Arc::new(Mutex::new(())),
            swift_format_on_cmd_s_active,
        }
    }

    pub fn set_ai_features_active(&mut self, ai_features_active: bool) {
        self.ai_features_active = ai_features_active;
    }

    pub fn set_swift_format_on_cmd_s_active(&mut self, active: bool) {
        self.swift_format_on_cmd_s_active = active;
    }

    pub fn run_features(
        &mut self,
        editor_window_uid: EditorWindowUid,
        trigger: CoreEngineTrigger,
        text_range: Option<&TextRange>,
    ) -> Result<(), CoreEngineError> {
        self.schedule_feature_procedures(&trigger, editor_window_uid);

        let finished_recv = self.compute_code_doc_updates(text_range, editor_window_uid, &trigger);

        if finished_recv.is_some() {
            self.finished_code_doc_update_task_recv = finished_recv;
        }

        self.process_features_schedule();

        Ok(())
    }

    fn process_features_schedule(&mut self) {
        tauri::async_runtime::spawn({
            let feature_procedures_schedule = self.feature_procedures_schedule.clone();
            let features = self.features.clone();
            let code_documents = self.code_documents.clone();
            let code_doc_finished_recv = self.finished_code_doc_update_task_recv.take();
            let awaiting_arc = self.awaiting_code_doc_update_task.clone();

            async move {
                if let Some(finished_recv) = code_doc_finished_recv {
                    let _ = awaiting_arc.lock();
                    if let Ok(code_doc_update) = finished_recv.await {
                        match code_doc_update {
                            CodeDocUpdate::Finished => {
                                Self::process_features(
                                    feature_procedures_schedule.clone(),
                                    features,
                                    code_documents,
                                );
                            }
                            CodeDocUpdate::Aborded => {
                                // Syntax tree parsing task aborded.
                            }
                        }
                    }
                } else {
                    if !awaiting_arc.is_locked() {
                        Self::process_features(
                            feature_procedures_schedule,
                            features,
                            code_documents,
                        );
                    }
                }
            }
        });
    }

    fn process_features(
        feature_procedures_schedule: Arc<Mutex<FeatureProcedureSchedule>>,
        features: Arc<Mutex<HashMap<FeatureKind, Arc<Mutex<Feature>>>>>,
        code_documents: Arc<Mutex<HashMap<usize, CodeDocument>>>,
    ) {
        for (feature_kind, trigger, procedure, window_uid) in
            feature_procedures_schedule.lock().values()
        {
            if let (Some(feature), Some(code_doc)) = (
                features.lock().get_mut(feature_kind),
                code_documents.lock().get(&window_uid),
            ) {
                Self::process_single_feature(
                    trigger,
                    procedure.to_owned(),
                    code_doc.to_owned(),
                    feature,
                );
            }
        }

        feature_procedures_schedule.lock().clear();
    }

    fn process_single_feature(
        trigger: &CoreEngineTrigger,
        procedure: FeatureProcedure,
        code_doc: CodeDocument,
        feature: &mut Arc<Mutex<Feature>>,
    ) {
        tauri::async_runtime::spawn({
            let trigger = trigger.clone();
            let feature = feature.clone();
            async move {
                match procedure {
                    FeatureProcedure::LongRunning => feature
                        .lock()
                        .compute_long_running(code_doc, &trigger, None),
                    FeatureProcedure::ShortRunning => {
                        feature.lock().compute_short_running(code_doc, &trigger)
                    }
                }
            }
        });
    }

    fn schedule_feature_procedures(
        &mut self,
        trigger: &CoreEngineTrigger,
        window_uid: EditorWindowUid,
    ) {
        let mut feature_procedures_schedule = self.feature_procedures_schedule.lock();

        for feature_kind in FeatureKind::iter() {
            match feature_kind {
                FeatureKind::Formatter => {
                    if !self.swift_format_on_cmd_s_active {
                        continue;
                    }
                }
                _ => {}
            }

            if feature_kind.requires_ai() && !self.ai_features_active {
                continue;
            }

            if let Some(procedure) = feature_kind.should_compute(trigger) {
                feature_procedures_schedule.insert(
                    hash_trigger_and_feature(trigger, &feature_kind),
                    (
                        feature_kind.to_owned(),
                        trigger.to_owned(),
                        procedure,
                        window_uid,
                    ),
                );
            }
        }
    }

    fn reset_cancellation_channel(&mut self) -> oneshot::Receiver<&'static ()> {
        if let Some(sender) = self.cancel_code_doc_update_task_send.take() {
            // Cancel previous task if it exists.
            _ = sender.send(&());
        }

        let (send, recv) = oneshot::channel();
        self.cancel_code_doc_update_task_send = Some(send);
        recv
    }

    fn compute_code_doc_updates(
        &mut self,
        text_range: Option<&TextRange>,
        window_uid: EditorWindowUid,
        trigger: &CoreEngineTrigger,
    ) -> Option<Receiver<&'static CodeDocUpdate>> {
        let (ast_compute_send, ast_compute_recv) = oneshot::channel();
        let (code_doc_update_send, code_doc_update_recv) = oneshot::channel();

        match trigger {
            CoreEngineTrigger::OnTextContentChange | CoreEngineTrigger::OnTextSelectionChange => {
                let cancel_recv = self.reset_cancellation_channel();

                tauri::async_runtime::spawn({
                    let code_documents = self.code_documents.clone();
                    let text_range = text_range.cloned();

                    async move {
                        // Spin up task to compute syntax tree
                        tauri::async_runtime::spawn({
                            let code_documents = code_documents.clone();

                            let previous_tree = {
                                let mut code_docs = code_documents.lock();
                                let code_doc = code_docs.get_mut(&window_uid);
                                match code_doc {
                                    Some(code_doc) => code_doc.syntax_tree().cloned(),
                                    None => return,
                                }
                            };

                            async move {
                                if let Err(e) = Self::compute_abstract_syntax_tree(
                                    code_documents,
                                    window_uid,
                                    previous_tree,
                                    ast_compute_send,
                                )
                                .await
                                {
                                    debug!("Error in compute_abstract_syntax_tree: {:?}", e);
                                }
                            }
                        });

                        tokio::select! {
                            recv_res = ast_compute_recv => {
                                match recv_res {
                                    Ok(tree_option) => {
                                        _ = Self::update_code_document(
                                            code_documents,
                                            window_uid,
                                            tree_option,
                                            text_range,
                                        );
                                        _ = code_doc_update_send.send(&CodeDocUpdate::Finished);
                                    }
                                    Err(_) => {
                                        // Channel closed
                                    },
                                }

                            }
                            _ = cancel_recv => {
                                _ = code_doc_update_send.send(&CodeDocUpdate::Aborded);
                            }
                        }
                    }
                });
            }
            _ => {
                return None;
            }
        };

        Some(code_doc_update_recv)
    }

    pub async fn compute_abstract_syntax_tree(
        code_documents: Arc<Mutex<HashMap<EditorWindowUid, CodeDocument>>>,
        window_uid: EditorWindowUid,
        previous_ast: Option<SwiftSyntaxTree>,
        mut sender: oneshot::Sender<Option<SwiftSyntaxTree>>,
    ) -> Result<(), CoreEngineError> {
        let code_text_u16;
        {
            // Check if code doc text has updated
            let docs = code_documents.lock();
            let code_doc = docs
                .get(&window_uid)
                .ok_or(CoreEngineError::CodeDocNotFound(window_uid))?;

            let current_ast = code_doc.syntax_tree();

            let code_text = get_textarea_content(&GetVia::Hash(window_uid))
                .map_err(|e| CoreEngineError::GenericError(e.into()))?;

            code_text_u16 = XcodeText::from_str(&code_text);

            if let Some(current_ast) = current_ast {
                if code_text_u16 == *current_ast.text_content() {
                    // No change in text, return previous tree
                    _ = sender.send(None);
                    return Ok(());
                }
            }
        }

        // Recompute AST because code text has changed
        Ok(tokio::select! {
            syntax_tree = SwiftSyntaxTree::from_XcodeText(code_text_u16, previous_ast) => {
                match syntax_tree {
                    Ok(tree) => {
                        _ = sender.send(Some(tree));
                    },
                    Err(e) => {
                        error!("Error while computing AST: {}", e);
                    },
                }
            }
            _ = sender.closed() => {
                // OneShot Channel closed
            }
        })
    }

    fn update_code_document(
        code_documents: Arc<Mutex<HashMap<EditorWindowUid, CodeDocument>>>,
        window_uid: EditorWindowUid,
        syntax_tree: Option<SwiftSyntaxTree>,
        text_range: Option<TextRange>,
    ) -> Result<(), CoreEngineError> {
        let mut code_docs_arc = code_documents.lock();
        let code_doc = code_docs_arc
            .get_mut(&window_uid)
            .ok_or(CoreEngineError::CodeDocNotFound(window_uid))?;

        let file_path = get_textarea_file_path(&GetVia::Hash(window_uid)).ok();

        if let Some(path) = file_path.as_ref() {
            if code_doc.file_path().to_owned() != Some(path.clone()) {
                log_list_of_module_names(path.clone());
            }
        }

        if let Some(syntax_tree) = syntax_tree {
            code_doc.update_code_text(syntax_tree, file_path);
        }

        if let Some(text_range) = text_range {
            code_doc.update_selected_text_range(text_range);
        }

        Ok(())
    }

    pub fn add_code_document(&mut self, editor_pid: i32, editor_window_uid: EditorWindowUid) {
        // check if code document is already contained in list of documents
        if self.code_documents.lock().get(&editor_window_uid).is_none() {
            let new_code_doc = CodeDocument::new(&EditorWindowProps {
                window_uid: editor_window_uid,
                pid: editor_pid,
            });

            self.code_documents
                .lock()
                .insert(editor_window_uid, new_code_doc);
        }
    }

    pub fn remove_code_document(
        &mut self,
        editor_window_uid: EditorWindowUid,
    ) -> Result<(), CoreEngineError> {
        if self
            .code_documents
            .lock()
            .remove(&editor_window_uid)
            .is_none()
        {
            Err(CoreEngineError::CodeDocNotFound(editor_window_uid))
        } else {
            Ok(())
        }
    }

    pub fn reset_features(&mut self) {
        for feature in &mut self.features.lock().values() {
            _ = feature.lock().reset();
        }

        *self.code_documents.lock() = HashMap::new();
    }

    pub fn start_core_engine_listeners(core_engine: &Arc<Mutex<CoreEngine>>) {
        xcode_listener(&core_engine);
        user_interaction_listener(&core_engine);
    }
}
