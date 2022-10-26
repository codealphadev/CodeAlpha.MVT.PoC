use parking_lot::Mutex;
use std::sync::Arc;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::EditorTextareaContentChangedMessage,
};

pub fn on_text_content_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    content_changed_msg: &EditorTextareaContentChangedMessage,
) -> Result<(), CoreEngineError> {
    let core_engine = &mut core_engine_arc.lock();

    core_engine.add_code_document(content_changed_msg.pid, content_changed_msg.window_uid);

    core_engine.run_features(
        content_changed_msg.window_uid,
        CoreEngineTrigger::OnTextContentChange,
        None,
    )
}
