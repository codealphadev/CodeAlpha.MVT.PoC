use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle, ax_interaction::AXEventApp, utils::messaging::ChannelList,
    window_controls::WindowManager,
};

use super::handlers::{
    on_activated_app, on_deactivate_app, on_focused_app_window, on_move_app_window,
};

pub fn app_listener(window_manager: &Arc<Mutex<WindowManager>>) {
    let window_manager_move_copy = (window_manager).clone();
    app_handle().listen_global(ChannelList::AXEventApp.to_string(), move |msg| {
        let axevent_app: AXEventApp = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_app {
            AXEventApp::AppWindowFocused(msg) => {
                on_focused_app_window(&window_manager_move_copy, &msg);
            }
            AXEventApp::AppWindowMoved(msg) => {
                on_move_app_window(&window_manager_move_copy, &msg);
            }
            AXEventApp::AppUIElementFocused(_) => {
                // Do Nothing here
            }
            AXEventApp::AppContentActivationChange(_) => {
                // Do Nothing here -- needs refactoring, ContentWindow now "MainWindow"
            }
            AXEventApp::AppActivated(msg) => {
                on_activated_app(&window_manager_move_copy, &msg);
                // Do Nothing
            }
            AXEventApp::AppDeactivated(msg) => {
                on_deactivate_app(&window_manager_move_copy, &msg);
            }
        }
    });
}
