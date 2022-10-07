use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::input_device::MouseClickMessage, window_controls::TrackingAreasManager,
};

pub fn on_mouse_clicked(
    tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>,
    click_msg: &MouseClickMessage,
) {
    let mut tracking_area_manager = tracking_area_manager_arc.lock();

    tracking_area_manager.track_mouse_click(&click_msg);
}
