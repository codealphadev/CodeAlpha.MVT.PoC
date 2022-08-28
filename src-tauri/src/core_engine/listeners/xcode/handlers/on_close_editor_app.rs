use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::core_engine::CoreEngine;

pub fn on_close_editor_app(core_engine_arc: &Arc<Mutex<CoreEngine>>) {
    let mut core_engine = core_engine_arc.lock();

    *core_engine.code_documents().lock() = HashMap::new();

    core_engine.reset_features();
}
