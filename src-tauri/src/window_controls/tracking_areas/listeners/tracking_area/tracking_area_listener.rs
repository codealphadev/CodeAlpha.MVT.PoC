use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    utils::messaging::ChannelList,
    window_controls::{EventTrackingArea, TrackingAreasManager},
};

pub fn tracking_area_listener(tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>) {
    app_handle().listen_global(ChannelList::EventTrackingAreas.to_string(), {
        let tracking_area_manager_arc = (tracking_area_manager_arc).clone();
        move |msg| {
            let event_tracking_areas: EventTrackingArea =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            let mut tracking_area_manager = tracking_area_manager_arc.lock();

            match event_tracking_areas {
                EventTrackingArea::Add(msg) => {
                    tracking_area_manager.add_tracking_areas(msg);
                }
                EventTrackingArea::Remove(msg) => {
                    tracking_area_manager.remove_tracking_areas(msg);
                }
                EventTrackingArea::Reset() => {
                    tracking_area_manager.reset_tracking_areas();
                }
                EventTrackingArea::Update(msg) => {
                    tracking_area_manager.update_tracking_areas(msg);
                }
                EventTrackingArea::Replace(msg) => {
                    tracking_area_manager.replace_tracking_areas(msg);
                }
            }
        }
    });
}
