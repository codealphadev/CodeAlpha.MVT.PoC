use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::EditorTextareaZoomedMessage,
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

    core_engine.run_features(
        zoomed_msg.window_uid,
        &CoreEngineTrigger::OnViewportDimensionsChange,
    );

    Some(())
}
