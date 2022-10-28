use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::EditorTextareaZoomedMessage,
};

pub fn on_editor_textarea_zoomed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    zoomed_msg: &EditorTextareaZoomedMessage,
) -> Result<(), CoreEngineError> {
    let core_engine = &mut core_engine_arc.lock();

    core_engine.handle_trigger(
        zoomed_msg.window_uid,
        CoreEngineTrigger::OnViewportDimensionsChange,
    )
}
