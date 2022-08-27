use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::{get_viewport_frame, models::editor::EditorWindowResizedMessage, GetVia},
    core_engine::CoreEngine,
};

pub fn on_editor_window_resized(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    resized_msg: &EditorWindowResizedMessage,
) -> Option<()> {
    let core_engine = core_engine_arc.lock();

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return None;
    }

    let code_documents = core_engine.code_documents().lock();

    if let Some(code_doc) = code_documents.get_mut(&resized_msg.uielement_hash) {
        code_doc.update_editor_window_viewport(get_viewport_frame(&GetVia::Current).ok()?);
        code_doc.update_docs_gen_annotation_visualization();
    }
    Some(())
}
