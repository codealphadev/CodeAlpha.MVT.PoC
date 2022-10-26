use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::EditorWindowResizedMessage,
};

pub fn on_editor_window_resized(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    resized_msg: &EditorWindowResizedMessage,
) -> Result<(), CoreEngineError> {
    let mut core_engine = core_engine_arc.lock();

    core_engine.run_features(
        resized_msg.window_uid,
        CoreEngineTrigger::OnViewportDimensionsChange,
        None,
    )
}
