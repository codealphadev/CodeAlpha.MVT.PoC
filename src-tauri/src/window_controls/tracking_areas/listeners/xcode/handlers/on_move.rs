use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::editor::EditorWindowMovedMessage,
    window_controls::TrackingAreasManager,
};

pub fn on_move_editor_window(
    tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>,
    moved_msg: &EditorWindowMovedMessage,
) {
    let mut tracking_area_manager = tracking_area_manager_arc.lock();

    tracking_area_manager.move_tracking_areas(&moved_msg.origin_delta, moved_msg.window_uid);
}
