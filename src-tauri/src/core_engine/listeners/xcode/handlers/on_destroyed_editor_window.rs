use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, CoreEngine},
    platform::macos::models::editor::EditorWindowDestroyedMessage,
};

pub fn on_editor_window_destroyed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    destroyed_msg: &EditorWindowDestroyedMessage,
) -> Result<(), CoreEngineError> {
    let core_engine = &mut core_engine_arc.lock();
    let code_documents = &mut core_engine.code_documents().lock();

    _ = code_documents
        .remove(&destroyed_msg.window_uid)
        .ok_or(CoreEngineError::CodeDocNotFound(destroyed_msg.window_uid))?;

    Ok(())
}
