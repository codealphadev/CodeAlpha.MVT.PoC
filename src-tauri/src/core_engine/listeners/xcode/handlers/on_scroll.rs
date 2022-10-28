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
    let core_engine = &mut core_engine_arc.lock();

    let focused_window = get_focused_window()?;

    core_engine.handle_trigger(focused_window, CoreEngineTrigger::OnVisibleTextRangeChange)
}
