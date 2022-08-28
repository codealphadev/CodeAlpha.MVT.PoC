use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{features::CoreEngineTrigger, CoreEngine},
    platform::macos::{get_viewport_frame, models::editor::EditorTextareaZoomedMessage, GetVia},
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

    if let Some(code_doc) = code_documents.get_mut(&zoomed_msg.window_uid) {
        code_doc.update_editor_window_viewport(get_viewport_frame(&GetVia::Current).ok()?);
        code_doc.compute_rule_visualizations();
        core_engine.run_features(code_doc, &CoreEngineTrigger::OnViewportDimensionsChange);
    }

    Some(())
}
