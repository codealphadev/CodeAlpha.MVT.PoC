use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    window_controls::models::TrackingAreaClickedMessage,
};

pub fn on_tracking_area_clicked(
    clicked_msg: TrackingAreaClickedMessage,
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
) -> Result<(), CoreEngineError> {
    let mut core_engine = core_engine_arc.lock();

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return Ok(());
    }

    core_engine.run_features(
        clicked_msg.editor_window_uid,
        &CoreEngineTrigger::OnTrackingAreaClicked(clicked_msg),
    )
}
