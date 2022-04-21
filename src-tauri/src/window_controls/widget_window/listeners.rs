use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tauri::Manager;

use crate::ax_interaction::{
    AXEventApp, AXEventXcode, AX_EVENT_APP_CHANNEL, AX_EVENT_XCODE_CHANNEL,
};

use super::{
    handler_ax_events_app::{on_move_app_window, on_toggle_content_window},
    handler_ax_events_xcode::{
        on_activate_editor_app, on_close_editor_app, on_deactivate_editor_app,
        on_editor_ui_element_focus_change, on_move_editor_window, on_resize_editor_window,
    },
    widget_window::HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS,
    WidgetWindow,
};

pub fn register_listener_xcode(
    app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
) {
    let widget_props_move_copy = (widget_props).clone();
    app_handle.listen_global(AX_EVENT_XCODE_CHANNEL, move |msg| {
        let mut widget_props_locked = widget_props_move_copy.lock().unwrap();

        let axevent_xcode: AXEventXcode = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_xcode {
            AXEventXcode::EditorWindowResized(msg) => {
                on_resize_editor_window(&mut *widget_props_locked, &msg);
            }
            AXEventXcode::EditorWindowMoved(msg) => {
                on_move_editor_window(&mut *widget_props_locked, &msg);
            }
            AXEventXcode::EditorUIElementFocused(msg) => {
                on_editor_ui_element_focus_change(&mut *widget_props_locked, &msg);
            }
            AXEventXcode::EditorAppActivated(msg) => {
                on_activate_editor_app(&mut *widget_props_locked, &msg)
            }
            AXEventXcode::EditorAppDeactivated(msg) => {
                on_deactivate_editor_app(&mut *widget_props_locked, &msg)
            }
            AXEventXcode::EditorAppClosed(msg) => {
                on_close_editor_app(&mut *widget_props_locked, &msg)
            }
            _ => {}
        }
    });
}

pub fn register_listener_app(
    app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
) {
    let widget_props_move_copy = (widget_props).clone();
    app_handle.listen_global(AX_EVENT_APP_CHANNEL, move |msg| {
        let mut widget_props_locked = widget_props_move_copy.lock().unwrap();

        let axevent_app: AXEventApp = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match &axevent_app {
            AXEventApp::AppWindowFocused(msg) => {
                (*widget_props_locked).currently_focused_app_window = Some(msg.window);
            }
            AXEventApp::AppWindowMoved(msg) => {
                on_move_app_window(&mut *widget_props_locked, &msg);
            }
            AXEventApp::AppUIElementFocused(_) => {
                // TODO: Do nothing
            }
            AXEventApp::AppActivated(_) => {
                (*widget_props_locked).is_app_focused = true;
            }
            AXEventApp::AppDeactivated(_) => {
                (*widget_props_locked).is_app_focused = false;

                // Reset hide timer after which the widget should be displayed again
                (*widget_props_locked).delay_hide_until_instant =
                    Instant::now() + Duration::from_millis(HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS);
            }
            AXEventApp::AppContentActivationChange(msg) => {
                on_toggle_content_window(&mut *widget_props_locked, msg);
            }
            AXEventApp::None => {}
        }
    });
}
