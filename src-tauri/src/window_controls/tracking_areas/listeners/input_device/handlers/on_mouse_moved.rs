use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::input_device::MouseMovedMessage, window_controls::TrackingAreasManager,
};

pub fn on_mouse_moved(
    tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>,
    move_msg: &MouseMovedMessage,
) {
    let mut tracking_area_manager = tracking_area_manager_arc.lock();

    tracking_area_manager
        .track_mouse_position(move_msg.cursor_position.x, move_msg.cursor_position.y);
}
