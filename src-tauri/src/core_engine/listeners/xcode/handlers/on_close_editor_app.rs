use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::core_engine::CoreEngine;

pub fn on_close_editor_app(core_engine_arc: &Arc<Mutex<CoreEngine>>) {
    let core_engine = core_engine_arc.lock();

    *core_engine.code_documents().lock() = HashMap::new();
}
