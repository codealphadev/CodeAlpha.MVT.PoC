use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{rules::RuleType, CodeDocument};

pub type CodeDocumentsArcMutex = Arc<Mutex<HashMap<uuid::Uuid, CodeDocument>>>;

pub struct CoreEngine {
    pub app_handle: tauri::AppHandle,

    /// List of open code documents.
    code_documents: CodeDocumentsArcMutex,

    /// It's a way to keep track of what feature is currently active.
    active_feature: RuleType,

    /// Identifier indicating if the app is currently active and supposed to give suggestions
    engine_active: bool,
}

impl CoreEngine {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        Self {
            app_handle: app_handle.clone(),
            code_documents: Arc::new(Mutex::new(HashMap::new())),
            active_feature: RuleType::None,
            engine_active: false,
        }
    }

    pub fn active_feature(&self) -> RuleType {
        self.active_feature.clone()
    }

    pub fn engine_active(&self) -> bool {
        self.engine_active
    }

    pub fn code_documents(&mut self) -> &mut CodeDocumentsArcMutex {
        &mut self.code_documents
    }

    pub fn set_engine_active(&mut self, engine_active: Option<bool>) {
        if let Some(engine_active) = engine_active {
            self.engine_active = engine_active;
        }
    }

    pub fn set_active_feature(&mut self, active_feature: Option<RuleType>) {
        if let Some(active_feature) = active_feature {
            self.active_feature = active_feature;
        }
    }
}
