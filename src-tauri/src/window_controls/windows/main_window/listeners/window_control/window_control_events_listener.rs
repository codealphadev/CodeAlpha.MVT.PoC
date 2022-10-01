use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    utils::messaging::ChannelList,
    window_controls::{config::AppWindow, events::EventWindowControls, windows::MainWindow},
};

use super::handlers::{on_hide_app_window, on_show_app_window};

pub fn window_control_events_listener(main_window: &Arc<Mutex<MainWindow>>) {
    let main_window_move_copy = (main_window).clone();
    app_handle().listen_global(ChannelList::EventWindowControls.to_string(), move |msg| {
        let event_window_controls: EventWindowControls =
            serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match event_window_controls {
            EventWindowControls::AppWindowHide(msg) => {
                on_hide_app_window(&main_window_move_copy, &msg);
            }
            EventWindowControls::AppWindowShow(msg) => {
                on_show_app_window(&main_window_move_copy, &msg);
            }
            EventWindowControls::RebindMainAndWidget => {
                _ = rebind_main_and_widget_window();
            }
            _ => {
                // Do Nothing here
            }
        }
    });
}

use cocoa::{appkit::NSWindowOrderingMode, base::id};
use objc::{msg_send, sel, sel_impl};

fn rebind_main_and_widget_window() -> Option<()> {
    let widget_tauri_window = app_handle().get_window(&AppWindow::Widget.to_string())?;

    let main_tauri_window = app_handle().get_window(&AppWindow::Main.to_string())?;
    if let (Ok(parent_ptr), Ok(child_ptr)) = (
        widget_tauri_window.ns_window(),
        main_tauri_window.ns_window(),
    ) {
        unsafe {
            let _: () = msg_send![parent_ptr as id, addChildWindow: child_ptr as id ordered: NSWindowOrderingMode::NSWindowBelow];
        }
    }

    Some(())
}
