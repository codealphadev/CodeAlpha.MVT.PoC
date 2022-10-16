use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{
        core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine, TextRange,
    },
    platform::macos::models::editor::EditorTextareaSelectedTextChangedMessage,
};

pub fn on_selected_text_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    msg: &EditorTextareaSelectedTextChangedMessage,
) -> Result<(), CoreEngineError> {
    let core_engine = &mut core_engine_arc.lock();

    let text_changed;
    {
        let code_documents = &mut core_engine.code_documents().lock();

        let code_doc = code_documents
            .get_mut(&msg.window_uid)
            .ok_or(CoreEngineError::CodeDocNotFound(msg.window_uid))?;

        text_changed = code_doc.set_selected_text_range_and_get_if_text_changed(
            &TextRange {
                index: msg.index,
                length: msg.length,
            },
            true,
        );
    }

    core_engine.run_features(msg.window_uid, &CoreEngineTrigger::OnTextSelectionChange)?;
    if text_changed {
        core_engine.run_features(msg.window_uid, &CoreEngineTrigger::OnTextContentChange)?;
    }
    Ok(())
}
