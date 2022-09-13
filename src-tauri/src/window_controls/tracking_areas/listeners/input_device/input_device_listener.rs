use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    platform::macos::EventInputDevice,
    utils::messaging::ChannelList,
    window_controls::{config::AppWindow, TrackingAreasManager},
};

use super::handlers::{on_mouse_clicked, on_mouse_moved};

pub fn input_devices_listener(tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>) {
    app_handle().listen_global(ChannelList::EventInputDevice.to_string(), {
        let tracking_area_manager_arc = (tracking_area_manager_arc).clone();

        move |msg| {
            // Only process mouse events if our app is currently shown.
            if !check_app_visible() {
                return;
            }

            let event_input_device: EventInputDevice =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_input_device {
                EventInputDevice::MouseMoved(msg) => {
                    on_mouse_moved(&tracking_area_manager_arc, &msg);
                }
                EventInputDevice::MouseClick(msg) => {
                    on_mouse_clicked(&tracking_area_manager_arc, &msg);
                }
            }
        }
    });
}

fn check_app_visible() -> bool {
    use strum::IntoEnumIterator;

    for app_window in AppWindow::iter() {
        if let Some(window) = app_handle().get_window(&app_window.to_string()) {
            if let Ok(visible) = window.is_visible() {
                if visible {
                    return true;
                }
            }
        }
    }

    false
}
