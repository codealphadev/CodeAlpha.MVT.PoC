use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::{
        get_focused_window,
        models::editor::{EditorTextareaScrolledMessage, EditorTextareaScrollingFinishedMessage},
    },
};

pub fn on_editor_textarea_scrolling_finished(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    _: &EditorTextareaScrollingFinishedMessage,
) -> Result<(), CoreEngineError> {
    let core_engine = &mut core_engine_arc.lock();

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return Ok(());
    }
    let focused_window = get_focused_window()?;

    core_engine.run_features(focused_window, &CoreEngineTrigger::OnScrollingFinished);
    Ok(())
}
