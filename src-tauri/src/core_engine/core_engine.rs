use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{app_handle, CORE_ENGINE_ACTIVE_AT_STARTUP};

use super::{
    features::{
        BracketHighlight, CoreEngineTrigger, DocsGenerator, Feature, FeatureBase, SwiftFormatter,
    },
    listeners::{user_interaction::user_interaction_listener, xcode::xcode_listener},
    CodeDocument,
};

pub type WindowUid = usize;

pub type CodeDocumentsArcMutex = Arc<Mutex<HashMap<WindowUid, CodeDocument>>>;

#[derive(thiserror::Error, Debug)]
pub enum CoreEngineError {
    #[error("There exists no CodeDocument with window_uid {0}.")]
    CodeDocNotFound(WindowUid),
    #[error("Something went wrong.")]
    GenericError(#[source] anyhow::Error),
}

pub struct CoreEngine {
    pub app_handle: tauri::AppHandle,

    /// List of open code documents.
    code_documents: CodeDocumentsArcMutex,

    /// Features include bracket highlighting, docs generation and formatters.
    features: Vec<Feature>,

    /// Identifier indicating if the app is currently active and supposed to give suggestions
    engine_active: bool,
}

impl CoreEngine {
    pub fn new() -> Self {
        Self {
            app_handle: app_handle(),
            code_documents: Arc::new(Mutex::new(HashMap::new())),
            engine_active: CORE_ENGINE_ACTIVE_AT_STARTUP,
            features: vec![
                Feature::BracketHighlighting(BracketHighlight::new()),
                Feature::Formatter(SwiftFormatter::new()),
                Feature::DocsGeneration(DocsGenerator::new()),
            ],
        }
    }

    pub fn engine_active(&self) -> bool {
        self.engine_active
    }

    pub fn code_documents(&mut self) -> &mut CodeDocumentsArcMutex {
        &mut self.code_documents
    }

    pub fn set_engine_active(&mut self, engine_active_status: bool) {
        self.engine_active = engine_active_status;

        if engine_active_status {
            // Activate features (currently nothing needs to be done)
            for feature in &mut self.features {
                feature.activate();
            }
        } else {
            // Activate features (currently nothing needs to be done)
            for feature in &mut self.features {
                feature.deactivate();
            }
        }
    }

    pub fn start_core_engine_listeners(core_engine: &Arc<Mutex<CoreEngine>>) {
        xcode_listener(&core_engine);
        user_interaction_listener(&core_engine);
    }

    pub fn run_features(&mut self, window_uid: WindowUid, trigger: &CoreEngineTrigger) {
        let code_documents = self.code_documents().lock();

        let code_doc = if let Some(code_document) = code_documents.get(&window_uid) {
            code_document
        } else {
            return;
        };

        self.compute_features(code_doc, trigger);
        self.update_feature_visualizations(code_doc, trigger);
    }

    fn compute_features(&mut self, code_doc: &CodeDocument, trigger: &CoreEngineTrigger) {
        for feature in &mut self.features {
            feature.compute(code_doc, trigger);
        }
    }

    fn update_feature_visualizations(
        &mut self,
        code_doc: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) {
        for feature in &mut self.features {
            feature.update_visualization(code_doc, trigger);
        }
    }

    pub fn reset_features(&mut self) {
        for feature in &mut self.features {
            feature.reset();
        }
    }
}
