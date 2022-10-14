use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::app::AppWindowMovedMessage, window_controls::TrackingAreasManager,
};

pub fn on_move_app_window(
    tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>,
    move_msg: &AppWindowMovedMessage,
) {
    let mut tracking_area_manager = tracking_area_manager_arc.lock();

    tracking_area_manager.update_app_window_tracking_areas(move_msg.window);
}
