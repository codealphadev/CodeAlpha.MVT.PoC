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

    core_engine.remove_code_document(destroyed_msg.window_uid)
}
