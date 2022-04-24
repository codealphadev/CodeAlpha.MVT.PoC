use std::sync::{Arc, Mutex};

use colored::Colorize;
use tauri::Manager;

use crate::{
    ax_interaction::{AXEventApp, AXEventXcode, AX_EVENT_APP_CHANNEL, AX_EVENT_XCODE_CHANNEL},
    window_controls::AppWindow,
};

use super::{
    handler_ax_events_app::{on_deactivate_app, on_move_app_window, on_toggle_content_window},
    handler_ax_events_xcode::{
        on_activate_editor_app, on_close_editor_app, on_deactivate_editor_app,
        on_editor_ui_element_focus_change, on_move_editor_window, on_resize_editor_window,
    },
    WidgetWindow,
};

pub fn register_listener_xcode(
    app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
) {
    let widget_props_move_copy = (widget_props).clone();
    let app_handle_move_copy = app_handle.clone();
    app_handle.listen_global(AX_EVENT_XCODE_CHANNEL, move |msg| {
        let axevent_xcode: AXEventXcode = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_xcode {
            AXEventXcode::EditorWindowResized(msg) => {
                on_resize_editor_window(&app_handle_move_copy, &widget_props_move_copy, &msg);
            }
            AXEventXcode::EditorWindowMoved(msg) => {
                on_move_editor_window(&app_handle_move_copy, &widget_props_move_copy, &msg);
            }
            AXEventXcode::EditorUIElementFocused(msg) => {
                on_editor_ui_element_focus_change(
                    &app_handle_move_copy,
                    &widget_props_move_copy,
                    &msg,
                );
            }
            AXEventXcode::EditorAppActivated(msg) => {
                on_activate_editor_app(&widget_props_move_copy, &msg)
            }
            AXEventXcode::EditorAppDeactivated(msg) => {
                on_deactivate_editor_app(&widget_props_move_copy, &msg)
            }
            AXEventXcode::EditorAppClosed(msg) => {
                on_close_editor_app(&widget_props_move_copy, &msg)
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
        let axevent_app: AXEventApp = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        println!(
            "AX_EVENT_APP_CHANNEL: {}",
            format!("{:?}", axevent_app).yellow()
        );

        match &axevent_app {
            AXEventApp::AppWindowFocused(msg) => {
                let widget_props = &mut *(widget_props_move_copy.lock().unwrap());
                widget_props.currently_focused_app_window = Some(msg.window);

                println!("AppWindowFocused: {:?}", msg);
            }
            AXEventApp::AppWindowMoved(msg) => {
                let widget_props = &mut *(widget_props_move_copy.lock().unwrap());
                on_move_app_window(widget_props, &msg);
            }
            AXEventApp::AppUIElementFocused(_) => {}
            AXEventApp::AppActivated(_) => {
                let widget_props = &mut *(widget_props_move_copy.lock().unwrap());
                widget_props.is_app_focused = true;
            }
            AXEventApp::AppDeactivated(msg) => {
                on_deactivate_app(&widget_props_move_copy, &msg);
            }
            AXEventApp::AppContentActivationChange(msg) => {
                let widget_props = &mut *(widget_props_move_copy.lock().unwrap());
                on_toggle_content_window(widget_props, msg);
            }
            AXEventApp::None => {}
        }

        let widget_props = &mut *(widget_props_move_copy.lock().unwrap());

        let app_handle = (*widget_props).app_handle.clone();
        let is_app_focused = (*widget_props).is_app_focused;
        let focused_window = (*widget_props).currently_focused_app_window;
        std::mem::drop(widget_props);

        if let Some(focused_window) = focused_window {
            if is_app_focused && focused_window == AppWindow::Widget {
                let widget_window = app_handle
                    .get_window(&AppWindow::Widget.to_string())
                    .unwrap();

                let _ = widget_window.start_dragging();
            }
        }
    });
}
