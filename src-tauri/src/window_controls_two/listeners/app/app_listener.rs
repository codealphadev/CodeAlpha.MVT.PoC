use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, ax_interaction::AXEventApp, utils::messaging::ChannelList,
    window_controls_two::WindowManager,
};

pub fn app_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    let window_manager_move_copy = (window_manager).clone();
    app_handle().listen_global(ChannelList::AXEventApp.to_string(), move |msg| {
        let axevent_app: AXEventApp = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_app {
            AXEventApp::AppWindowFocused(msg) => {
                // Do Nothing
            }
            AXEventApp::AppWindowMoved(msg) => {
                // Do Nothing
            }
            AXEventApp::AppUIElementFocused(msg) => {
                // Do Nothing
            }
            AXEventApp::AppContentActivationChange(msg) => {
                // Do Nothing
            }
            AXEventApp::AppActivated(msg) => {
                // Do Nothing
            }
            AXEventApp::AppDeactivated(msg) => {
                // Do Nothing
            }
        }
    });
}
