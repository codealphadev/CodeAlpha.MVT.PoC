use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::{get_focused_window, models::editor::EditorTextareaScrolledMessage},
};

pub fn on_editor_textarea_scrolled(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    _: &EditorTextareaScrolledMessage,
) -> Result<(), CoreEngineError> {
    if let Some(core_engine) = &mut core_engine_arc.try_lock() {
        // Checking if the engine is active. If not, it returns.
        if !core_engine.engine_active() {
            return Ok(());
        }
        println!("on_editor_textarea_scrolled");

        core_engine.run_features(
            get_focused_window()?,
            &CoreEngineTrigger::OnVisibleTextRangeChange,
        );
    } else {
        println!("Could not lock core engine");
    }

    Ok(())
}
