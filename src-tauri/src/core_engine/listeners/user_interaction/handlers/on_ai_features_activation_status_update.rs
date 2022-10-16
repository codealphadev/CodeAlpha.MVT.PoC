use std::sync::Arc;

use parking_lot::Mutex;

use crate::core_engine::{events::models::AiFeaturesStatusMessage, CoreEngine};

pub fn on_ai_features_activation_status_update(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    core_activation_status_msg: &AiFeaturesStatusMessage,
) {
    let mut core_engine = core_engine_arc.lock();

    core_engine.set_ai_features_active(core_activation_status_msg.ai_features_active);
}
