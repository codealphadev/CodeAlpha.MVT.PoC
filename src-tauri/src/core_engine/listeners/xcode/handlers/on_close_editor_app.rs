use std::sync::Arc;

use parking_lot::Mutex;

use crate::core_engine::{core_engine::CoreEngineError, CoreEngine};

pub fn on_close_editor_app(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
) -> Result<(), CoreEngineError> {
    core_engine_arc.lock().reset_features();

    Ok(())
}
