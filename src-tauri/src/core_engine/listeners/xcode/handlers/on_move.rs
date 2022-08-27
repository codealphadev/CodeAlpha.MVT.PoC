use std::sync::Arc;

use parking_lot::Mutex;

use crate::{ax_interaction::models::editor::EditorWindowMovedMessage, core_engine::CoreEngine};

pub fn on_editor_window_moved(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    moved_msg: &EditorWindowMovedMessage,
) {
    let core_engine = core_engine_arc.lock();

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return;
    }

    let code_documents = &mut core_engine.code_documents().lock();

    if let Some(code_doc) = code_documents.get_mut(&moved_msg.window_uid) {
        // TODO
    }
}
