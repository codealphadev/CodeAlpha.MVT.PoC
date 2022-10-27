use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::EditorTextareaSelectedTextChangedMessage,
};

pub fn on_selected_text_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    msg: &EditorTextareaSelectedTextChangedMessage,
) -> Result<(), CoreEngineError> {
    let core_engine = &mut core_engine_arc.lock();

    core_engine.handling_trigger(msg.window_uid, CoreEngineTrigger::OnTextSelectionChange)
}
