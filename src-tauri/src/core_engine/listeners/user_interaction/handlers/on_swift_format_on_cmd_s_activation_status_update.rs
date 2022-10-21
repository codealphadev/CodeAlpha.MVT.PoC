use std::sync::Arc;

use parking_lot::Mutex;

use crate::core_engine::{events::models::SwiftFormatOnCMDSMessage, CoreEngine};

pub fn on_swift_format_on_cmd_s_activation_status_update(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    swift_format_status_msg: &SwiftFormatOnCMDSMessage,
) {
    let mut core_engine = core_engine_arc.lock();

    core_engine.set_swift_format_on_cmd_s_active(swift_format_status_msg.active);
}
