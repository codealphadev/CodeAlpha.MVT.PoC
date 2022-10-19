use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use tauri::Manager;
use tracing::error;

use crate::{app_handle, app_state::CoreEngineStateCache, platform::macos::XcodeError};

use super::{
    annotations_manager::{AnnotationsManager, AnnotationsManagerTrait},
    features::{
        BracketHighlight, ComplexityRefactoring, CoreEngineTrigger, DocsGenerator, Feature,
        FeatureBase, FeatureError, SwiftFormatter,
    },
    listeners::{user_interaction::user_interaction_listener, xcode::xcode_listener},
    syntax_tree::SwiftSyntaxTree,
    CodeDocument,
};

pub type EditorWindowUid = usize;

pub type CodeDocumentsArcMutex = Arc<Mutex<HashMap<EditorWindowUid, CodeDocument>>>;

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

pub struct CoreEngine {
    pub app_handle: tauri::AppHandle,

    /// List of open code documents.
    code_documents: CodeDocumentsArcMutex,

    /// Features include bracket highlighting, docs generation and formatters.
    features: Vec<Arc<Mutex<Feature>>>,

    ai_features_active: bool,

    /// Annotations manager handles where to draw annotations on the code editor via the CodeOverlay window
    _annotations_manager: Arc<Mutex<AnnotationsManager>>,

    /// The swift parser in an Arc<Mutex> to allow it to be shared between threads -> TSParser does not implement Clone.
    parser_swift: Arc<Mutex<tree_sitter::Parser>>,
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

        Self {
            app_handle: app_handle(),
            code_documents: Arc::new(Mutex::new(HashMap::new())),
            ai_features_active,
            features: vec![
                Arc::new(Mutex::new(Feature::BracketHighlighting(
                    BracketHighlight::new(),
                ))),
                Arc::new(Mutex::new(Feature::Formatter(SwiftFormatter::new()))),
                Arc::new(Mutex::new(Feature::DocsGeneration(DocsGenerator::new()))),
                Arc::new(Mutex::new(Feature::ComplexityRefactoring(
                    ComplexityRefactoring::new(),
                ))),
            ],
            _annotations_manager: annotations_manager,
            parser_swift: SwiftSyntaxTree::parser_mutex(),
        }
    }

    pub fn swift_parser(&self) -> Arc<Mutex<tree_sitter::Parser>> {
        self.parser_swift.clone()
    }

    pub fn code_documents(&mut self) -> &mut CodeDocumentsArcMutex {
        &mut self.code_documents
    }

    pub fn set_ai_features_active(&mut self, ai_features_active: bool) {
        self.ai_features_active = ai_features_active;
    }

    pub fn run_features(
        &mut self,
        editor_window_uid: EditorWindowUid,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), CoreEngineError> {
        let code_documents = self.code_documents.lock();
        let code_doc = code_documents
            .get(&editor_window_uid)
            .ok_or(CoreEngineError::CodeDocNotFound(editor_window_uid))?;

        for feature in self.features.iter_mut() {
            tauri::async_runtime::spawn({
                let code_doc = code_doc.clone();
                let ai_features_active = self.ai_features_active;
                let feature = feature.clone();
                let trigger = trigger.clone();

                async move {
                    // Don't run features which require AI if AI is disabled
                    if !ai_features_active && feature.lock().requires_ai() {
                        _ = feature.lock().reset();
                        return;
                    }

                    if let Err(e) = feature.lock().compute(&code_doc, &trigger) {
                        error!(?e, ?feature, "Error in feature compute()")
                    }

                    if let Err(e) = feature.lock().update_visualization(&code_doc, &trigger) {
                        error!(?e, ?feature, "Error in feature update_visualization()")
                    }
                }
            });
        }

        Ok(())
    }

    pub fn reset_features(&mut self) {
        for feature in &mut self.features {
            _ = feature.lock().reset();
        }
    }

    pub fn start_core_engine_listeners(core_engine: &Arc<Mutex<CoreEngine>>) {
        xcode_listener(&core_engine);
        user_interaction_listener(&core_engine);
    }
}
