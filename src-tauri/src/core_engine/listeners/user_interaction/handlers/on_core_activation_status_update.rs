use std::sync::Arc;

use parking_lot::Mutex;

use crate::core_engine::{events::models::CoreActivationStatusMessage, CoreEngine};

pub fn on_core_activation_status_update(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    core_activation_status_msg: &CoreActivationStatusMessage,
) {
    let mut core_engine = core_engine_arc.lock();

    core_engine.set_engine_active(core_activation_status_msg.engine_active);
}
