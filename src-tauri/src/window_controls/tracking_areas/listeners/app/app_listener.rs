use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, platform::macos::AXEventApp, utils::messaging::ChannelList,
    window_controls::TrackingAreasManager,
};

use super::handlers::on_move_app_window;

pub fn appwindow_ax_listener(tracking_area_manager_arc: &Arc<Mutex<TrackingAreasManager>>) {
    app_handle().listen_global(ChannelList::AXEventApp.to_string(), {
        let tracking_area_manager_arc = (tracking_area_manager_arc).clone();

        move |msg| {
            let axevent_app: AXEventApp = serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match axevent_app {
                AXEventApp::AppWindowMoved(msg) => {
                    on_move_app_window(&tracking_area_manager_arc, &msg);
                }
                AXEventApp::AppWindowFocused(_) => {
                    // Do Nothing here
                }
                AXEventApp::AppUIElementFocused(_) => {
                    // Do Nothing here
                }
                AXEventApp::AppContentActivationChange(_) => {
                    // Do Nothing here -- needs refactoring, ContentWindow now "MainWindow"
                }
                AXEventApp::AppActivated(_) => {
                    // Do Nothing here
                }
                AXEventApp::AppDeactivated(_) => {
                    // Do Nothing
                }
            }
        }
    });
}
