use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::input_device::{ClickType, MouseButton, MouseClickMessage},
    window_controls::TrackingAreasManager,
};

pub fn on_mouse_clicked(
    tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>,
    click_msg: &MouseClickMessage,
) {
    let mut tracking_area_manager = tracking_area_manager_arc.lock();

    if click_msg.button == MouseButton::Left && click_msg.click_type == ClickType::Down {
        tracking_area_manager
            .track_mouse_click(click_msg.cursor_position.x, click_msg.cursor_position.y);
    }
}
