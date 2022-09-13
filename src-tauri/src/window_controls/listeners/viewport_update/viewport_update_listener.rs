use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    platform::macos::{models::viewport::ViewportPropertiesUpdateMessage, EventViewport},
    utils::messaging::ChannelList,
    window_controls::WindowManager,
};

pub fn viewport_update_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    app_handle().listen_global(ChannelList::EventViewport.to_string(), {
        let window_manager = (window_manager).clone();
        move |msg| {
            let event_viewport_update: EventViewport =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_viewport_update {
                EventViewport::XcodeViewportUpdate(msg) => {
                    on_viewport_update(&window_manager, &msg);
                }
            }
        }
    });
}

fn on_viewport_update(
    window_manager: &Arc<Mutex<WindowManager>>,
    update_msg: &ViewportPropertiesUpdateMessage,
) -> Option<()> {
    let window_manager = &mut window_manager.lock();

    window_manager.update_app_windows(update_msg)?;

    Some(())
}
