use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::EditorShortcutPressedMessage,
};

pub fn on_editor_shortcut_pressed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    msg: &EditorShortcutPressedMessage,
) -> Result<(), CoreEngineError> {
    let mut core_engine = core_engine_arc.lock();

    // Checking if the engine is active. If not, don't continue.
    if !core_engine.engine_active() {
        return Ok(());
    }

    core_engine.run_features(
        msg.window_uid,
        &CoreEngineTrigger::OnShortcutPressed(msg.clone()),
    )
}
