use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::{get_viewport_frame, models::editor::EditorTextareaZoomedMessage, GetVia},
    core_engine::CoreEngine,
};

pub fn on_editor_textarea_zoomed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    zoomed_msg: &EditorTextareaZoomedMessage,
) -> Option<()> {
    let core_engine = &mut core_engine_arc.lock();

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return None;
    }

    let code_documents = &mut core_engine.code_documents().lock();

    if let Some(code_doc) = code_documents.get_mut(&zoomed_msg.uielement_hash) {
        code_doc.update_editor_window_viewport(get_viewport_frame(&GetVia::Current).ok()?);
        code_doc.compute_rule_visualizations();
        code_doc.update_docs_gen_annotation_visualization();
    }

    Some(())
}
