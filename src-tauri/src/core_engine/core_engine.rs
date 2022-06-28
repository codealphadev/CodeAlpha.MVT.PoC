use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{
    listeners::{register_listener_user_interactions, register_listener_xcode},
    CodeDocument,
};

pub type CodeDocumentsArcMutex = Arc<Mutex<HashMap<uuid::Uuid, CodeDocument>>>;

pub struct CoreEngine {
    pub app_handle: tauri::AppHandle,

    /// List of open code documents.
    code_documents: CodeDocumentsArcMutex,

    /// Identifier indicating if the app is currently active and supposed to give suggestions
    engine_active: bool,
}

impl CoreEngine {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        Self {
            app_handle: app_handle.clone(),
            code_documents: Arc::new(Mutex::new(HashMap::new())),
            engine_active: true,
        }
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

    pub fn start_core_engine_listeners(
        app_handle: &tauri::AppHandle,
        core_engine: &Arc<Mutex<CoreEngine>>,
    ) {
        register_listener_xcode(app_handle, &core_engine);
        register_listener_user_interactions(app_handle, &core_engine);
    }
}
