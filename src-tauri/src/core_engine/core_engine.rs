use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{app_handle, CORE_ENGINE_ACTIVE_AT_STARTUP};

use super::{
    listeners::{user_interaction::user_interaction_listener, xcode::xcode_listener},
    CodeDocument,
};

pub type WindowUid = usize;

pub type CodeDocumentsArcMutex = Arc<Mutex<HashMap<WindowUid, CodeDocument>>>;

pub struct CoreEngine {
    pub app_handle: tauri::AppHandle,

    /// List of open code documents.
    code_documents: CodeDocumentsArcMutex,

    /// Identifier indicating if the app is currently active and supposed to give suggestions
    engine_active: bool,
}

impl CoreEngine {
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

        let code_documents = &mut self.code_documents().lock();

        if engine_active_status {
            // Activate features (currently nothing needs to be done)
        } else {
            // Deactivate features
            for code_document in code_documents.values_mut() {
                code_document.deactivate_features();
            }
        }
    }

    pub fn start_core_engine_listeners(core_engine: &Arc<Mutex<CoreEngine>>) {
        xcode_listener(&core_engine);
        user_interaction_listener(&core_engine);
    }
}
