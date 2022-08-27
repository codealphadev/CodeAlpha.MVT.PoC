use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorWindowDestroyedMessage, core_engine::CoreEngine,
};

pub fn on_editor_window_destroyed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    destroyed_msg: &EditorWindowDestroyedMessage,
) {
    let core_engine = &mut core_engine_arc.lock();
    let code_documents = &mut core_engine.code_documents().lock();

    let _ = &code_documents.remove(&destroyed_msg.uielement_hash);
}
