use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{app_handle, CORE_ENGINE_ACTIVE_AT_STARTUP};

use super::{
    listeners::{register_listener_user_interactions, register_listener_xcode},
    CodeDocument,
};

pub type UIElementHash = usize;

pub type CodeDocumentsArcMutex<'a> = Arc<Mutex<HashMap<UIElementHash, CodeDocument<'a>>>>;

pub struct CoreEngine<'a> {
    pub app_handle: tauri::AppHandle,

    /// List of open code documents.
    code_documents: CodeDocumentsArcMutex<'a>,

    /// Identifier indicating if the app is currently active and supposed to give suggestions
    engine_active: bool,
}

impl CoreEngine<'_> {
    pub fn new() -> Self {
        Self {
            app_handle: app_handle(),
            code_documents: Arc::new(Mutex::new(HashMap::new())),
            engine_active: CORE_ENGINE_ACTIVE_AT_STARTUP,
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

        let code_documents = &mut *(match self.code_documents().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        });

        if engine_active_status {
            // Activate features
            for code_document in code_documents.values_mut() {
                code_document.activate_features();
            }
        } else {
            // Deactivate features
            for code_document in code_documents.values_mut() {
                code_document.deactivate_features();
            }
        }
    }

    pub fn start_core_engine_listeners(core_engine: &Arc<Mutex<CoreEngine>>) {
        register_listener_xcode(&core_engine);
        register_listener_user_interactions(&core_engine);
    }
}
