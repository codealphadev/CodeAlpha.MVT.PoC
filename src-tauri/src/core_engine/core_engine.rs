use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use parking_lot::Mutex;
use strum::IntoEnumIterator;
use tauri::Manager;
use tokio::sync::oneshot::{self, Receiver};
use tracing::error;

use crate::{
    app_handle,
    app_state::CoreEngineStateCache,
    platform::macos::{
        get_selected_text_range, get_textarea_content, get_textarea_file_path, GetVia, XcodeError,
    },
};

use super::{
    annotations_manager::{AnnotationsManager, AnnotationsManagerTrait},
    features::{
        BracketHighlight, ComplexityRefactoring, CoreEngineTrigger, DocsGenerator, Feature,
        FeatureBase, FeatureError, FeatureKind, SwiftFormatter,
    },
    listeners::{user_interaction::user_interaction_listener, xcode::xcode_listener},
    log_list_of_module_names,
    syntax_tree::SwiftSyntaxTree,
    CodeDocument, EditorWindowProps, XcodeText,
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
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct CoreEngineProcedure {
    pub feature: FeatureKind,
    pub trigger: CoreEngineTrigger,
    pub window_uid: EditorWindowUid,
}

impl CoreEngineProcedure {
    pub fn new(
        feature: FeatureKind,
        trigger: CoreEngineTrigger,
        window_uid: EditorWindowUid,
    ) -> Self {
        Self {
            feature,
            trigger,
            window_uid,
        }
    }

    fn hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.trigger.hash(&mut hasher);
        self.feature.hash(&mut hasher);
        self.window_uid.hash(&mut hasher);
        hasher.finish()
    }
}

type CoreEngineProcedureSchedule = HashMap<u64, CoreEngineProcedure>;

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
    feature_procedures_schedule: Arc<Mutex<CoreEngineProcedureSchedule>>,

    /// A Sender that is used to cancel an ongoing code document update task
    cancel_code_doc_update_task_send: Option<oneshot::Sender<&'static ()>>,
    /// A receiver that gets notified when a code document update task is finished
    finished_code_doc_update_task_recv: Option<Receiver<&'static CodeDocUpdate>>,
    /// Is true if there is a procedures processor waiting to run once a code document has finished updating
    awaiting_code_doc_update_task: Arc<Mutex<bool>>,
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

        Self {
            app_handle: app_handle(),
            code_documents: Arc::new(Mutex::new(HashMap::new())),
            ai_features_active,
            features: Arc::new(Mutex::new(features)),
            _annotations_manager: annotations_manager,
            feature_procedures_schedule: Arc::new(Mutex::new(HashMap::new())),
            cancel_code_doc_update_task_send: None,
            finished_code_doc_update_task_recv: None,
            awaiting_code_doc_update_task: Arc::new(Mutex::new(false)),
            swift_format_on_cmd_s_active,
        }
    }

    pub fn set_ai_features_active(&mut self, ai_features_active: bool) {
        self.ai_features_active = ai_features_active;
    }

    pub fn set_swift_format_on_cmd_s_active(&mut self, active: bool) {
        self.swift_format_on_cmd_s_active = active;
    }

    pub fn handle_trigger(
        &mut self,
        editor_window_uid: EditorWindowUid,
        trigger: CoreEngineTrigger,
    ) -> Result<(), CoreEngineError> {
        self.schedule_feature_procedures(&trigger, editor_window_uid);

        if trigger == CoreEngineTrigger::OnTextContentChange
            || trigger == CoreEngineTrigger::OnTextSelectionChange
        {
            self.finished_code_doc_update_task_recv =
                Some(self.compute_code_doc_updates(editor_window_uid));
        }

        self.process_features_schedule();

        Ok(())
    }

    fn schedule_feature_procedures(
        &mut self,
        trigger: &CoreEngineTrigger,
        window_uid: EditorWindowUid,
    ) {
        let mut feature_procedures_schedule = self.feature_procedures_schedule.lock();

        for feature_kind in FeatureKind::iter() {
            if feature_kind == FeatureKind::Formatter && !self.swift_format_on_cmd_s_active {
                continue;
            }

            if feature_kind.requires_ai(&trigger) && !self.ai_features_active {
                continue;
            }

            if feature_kind.should_compute(trigger) {
                let procedure =
                    CoreEngineProcedure::new(feature_kind, trigger.to_owned(), window_uid);

                feature_procedures_schedule.insert(procedure.hash(), procedure);
            }
        }
    }

    fn process_features_schedule(&mut self) {
        tauri::async_runtime::spawn({
            let feature_procedures_schedule = self.feature_procedures_schedule.clone();
            let features = self.features.clone();
            let code_documents = self.code_documents.clone();
            let awaiting_code_doc_update_task = self.awaiting_code_doc_update_task.clone();

            let finished_code_doc_update_task_recv =
                if self.finished_code_doc_update_task_recv.is_some() {
                    *awaiting_code_doc_update_task.lock() = true;
                    self.finished_code_doc_update_task_recv.take()
                } else {
                    None
                };

            async move {
                if let Some(task_finished_recv) = finished_code_doc_update_task_recv {
                    // Waiting for an ongoing code_doc update to finish before proceeding.
                    if let Ok(code_doc_update) = task_finished_recv.await {
                        match code_doc_update {
                            CodeDocUpdate::Finished => {
                                Self::process_features(
                                    feature_procedures_schedule.clone(),
                                    features,
                                    code_documents,
                                );
                                *awaiting_code_doc_update_task.lock() = false;
                            }
                            CodeDocUpdate::Cancelled => {
                                // Syntax tree parsing task restarted.
                            }
                        }
                    }
                } else {
                    if *awaiting_code_doc_update_task.lock() == false {
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
        core_engine_procedures_schedule: Arc<Mutex<CoreEngineProcedureSchedule>>,
        features: Arc<Mutex<HashMap<FeatureKind, Arc<Mutex<Feature>>>>>,
        code_documents: Arc<Mutex<HashMap<usize, CodeDocument>>>,
    ) {
        for (_, core_engine_procedure) in core_engine_procedures_schedule.lock().drain() {
            if let (Some(feature), Some(code_doc)) = (
                features.lock().get_mut(&core_engine_procedure.feature),
                code_documents.lock().get(&core_engine_procedure.window_uid),
            ) {
                Self::process_single_feature(
                    &core_engine_procedure.trigger,
                    code_doc.to_owned(),
                    feature,
                );
            } else {
                error!(
                    ?core_engine_procedure,
                    "Feature or code document not found.",
                );
            }
        }
    }

    fn process_single_feature(
        trigger: &CoreEngineTrigger,
        code_doc: CodeDocument,
        feature: &mut Arc<Mutex<Feature>>,
    ) {
        tauri::async_runtime::spawn({
            let trigger = trigger.clone();
            let feature = feature.clone();
            async move {
                if let Err(e) = feature.lock().compute(code_doc, trigger) {
                    error!(?e, "Error while computing feature.");
                }
            }
        });
    }

    fn cancel_code_doc_update(&mut self) -> oneshot::Receiver<&'static ()> {
        if let Some(sender) = self.cancel_code_doc_update_task_send.take() {
            // Cancel previous code doc update task if it exists.
            _ = sender.send(&());
        }

        let (send, recv) = oneshot::channel();
        self.cancel_code_doc_update_task_send = Some(send);
        recv
    }

    fn compute_code_doc_updates(
        &mut self,
        window_uid: EditorWindowUid,
    ) -> Receiver<&'static CodeDocUpdate> {
        let (code_doc_update_send, code_doc_update_recv) = oneshot::channel();
        let (ast_compute_send, ast_compute_recv) = oneshot::channel();

        let cancel_recv = self.cancel_code_doc_update();

        tauri::async_runtime::spawn({
            let code_documents_arc = self.code_documents.clone();

            async move {
                // Spin up task to compute syntax tree
                tauri::async_runtime::spawn({
                    let code_documents_arc = code_documents_arc.clone();

                    async move {
                        if let Err(e) = Self::compute_abstract_syntax_tree(
                            code_documents_arc,
                            window_uid,
                            ast_compute_send,
                        )
                        .await
                        {
                            error!("Error in compute_abstract_syntax_tree: {:?}", e);
                        }
                    }
                });

                tokio::select! {
                    recv_res = ast_compute_recv => {
                        match recv_res {
                            Ok(tree_option) => {
                                _ = Self::update_code_document(
                                    code_documents_arc,
                                    window_uid,
                                    tree_option,
                                );
                                _ = code_doc_update_send.send(&CodeDocUpdate::Finished);
                            }
                            Err(_) => {
                                // Channel closed
                            },
                        }
                    }
                    _ = cancel_recv => {
                        _ = code_doc_update_send.send(&CodeDocUpdate::Cancelled);
                    }
                }
            }
        });

        return code_doc_update_recv;
    }

    pub async fn compute_abstract_syntax_tree(
        code_documents: Arc<Mutex<HashMap<EditorWindowUid, CodeDocument>>>,
        window_uid: EditorWindowUid,
        mut sender: oneshot::Sender<Option<SwiftSyntaxTree>>,
    ) -> Result<(), CoreEngineError> {
        let code_text_u16;
        let previous_ast;
        {
            let docs = code_documents.lock();
            let code_doc = docs
                .get(&window_uid)
                .ok_or(CoreEngineError::CodeDocNotFound(window_uid))?;

            previous_ast = code_doc.syntax_tree().cloned();
        }

        let code_text = get_textarea_content(&GetVia::Hash(window_uid))
            .map_err(|e| CoreEngineError::GenericError(e.into()))?;

        code_text_u16 = XcodeText::from_str(&code_text);

        if let Some(ref previous_ast) = previous_ast {
            if code_text_u16 == *previous_ast.text_content() {
                // No change in text
                _ = sender.send(None);
                return Ok(());
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
                        _ = sender.send(None);
                    },
                }
            }
            _ = sender.closed() => {}
        })
    }

    fn update_code_document(
        code_documents: Arc<Mutex<HashMap<EditorWindowUid, CodeDocument>>>,
        window_uid: EditorWindowUid,
        syntax_tree: Option<SwiftSyntaxTree>,
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

        let text_range = get_selected_text_range(&GetVia::Hash(window_uid))
            .map_err(|e| CoreEngineError::GenericError(e.into()))?;

        code_doc.update_selected_text_range(text_range);

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
