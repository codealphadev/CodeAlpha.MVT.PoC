use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::EditorWindowMovedMessage,
};

pub fn on_editor_window_moved(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    moved_msg: &EditorWindowMovedMessage,
) -> Result<(), CoreEngineError> {
    let mut core_engine = core_engine_arc.lock();

    core_engine.handling_trigger(moved_msg.window_uid, CoreEngineTrigger::OnViewportMove)
}
