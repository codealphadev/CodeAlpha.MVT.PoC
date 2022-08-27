use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::models::editor::EditorTextareaSelectedTextChangedMessage,
    core_engine::CoreEngine,
};

pub fn on_selected_text_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    msg: &EditorTextareaSelectedTextChangedMessage,
) {
    let core_engine = &mut core_engine_arc.lock();

    let core_engine_active_status = core_engine.engine_active();

    let code_documents = &mut core_engine.code_documents().lock();

    if let Some(code_doc) = code_documents.get_mut(&msg.window_uid) {
        code_doc.set_selected_text_range(msg.index, msg.length);
    }

    // Checking if the engine is active. If not, don't continue.
    if !core_engine_active_status {
        return;
    }
}
