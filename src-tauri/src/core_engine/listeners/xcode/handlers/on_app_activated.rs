use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::EditorAppActivatedMessage,
};

pub fn on_app_activated(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    activated_msg: &EditorAppActivatedMessage,
) -> Result<(), CoreEngineError> {
    let mut core_engine = core_engine_arc.lock();

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return Ok(());
    }

    core_engine.run_features(
        activated_msg.window_uid,
        &CoreEngineTrigger::OnTextSelectionChange,
    )
}
