use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{features::CoreEngineTrigger, CoreEngine},
    platform::macos::{get_focused_window, models::editor::EditorTextareaScrolledMessage},
};

pub fn on_editor_textarea_scrolled(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    _: &EditorTextareaScrolledMessage,
) {
    let core_engine = &mut core_engine_arc.lock();

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return;
    }

    let code_documents = &mut core_engine.code_documents().lock();

    if let Ok(focused_window) = get_focused_window() {
        if let Some(code_doc) = code_documents.get_mut(&focused_window) {
            code_doc.compute_rule_visualizations();
            core_engine.run_features(code_doc, &CoreEngineTrigger::OnVisibleTextRangeChange);
        }
    }
}
