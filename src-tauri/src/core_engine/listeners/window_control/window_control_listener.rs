use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, core_engine::CoreEngine, utils::messaging::ChannelList,
    window_controls::EventWindowControls,
};

use super::handlers::on_tracking_area_clicked;

pub fn window_control_listener(core_engine: &Arc<Mutex<CoreEngine>>) {
    app_handle().listen_global(ChannelList::EventWindowControls.to_string(), {
        let core_engine = (core_engine).clone();
        move |msg| {
            let event_window_controls: EventWindowControls =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_window_controls {
                EventWindowControls::TrackingAreaClicked(msg) => {
                    _ = on_tracking_area_clicked(msg, &core_engine);
                }
                _ => {}
            }
        }
    });
}
