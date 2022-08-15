use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::{
    ax_interaction::{AXEventApp, AXEventXcode},
    utils::messaging::ChannelList,
    window_controls::config::AppWindow,
};

use super::{
    handler_ax_events_app::{on_deactivate_app, on_move_app_window, on_toggle_content_window},
    handler_ax_events_xcode::{
        on_activate_editor_app, on_close_editor_app, on_deactivate_editor_app,
        on_editor_textarea_scrolled, on_editor_ui_element_focus_change, on_move_editor_window,
        on_resize_editor_window,
    },
    WidgetWindow,
};

pub fn register_listener_xcode(
    app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
) {
    let widget_props_move_copy = (widget_props).clone();
    let app_handle_move_copy = app_handle.clone();
    app_handle.listen_global(ChannelList::AXEventXcode.to_string(), move |msg| {
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
            AXEventXcode::EditorTextareaScrolled(msg) => {
                on_editor_textarea_scrolled(&app_handle_move_copy, &widget_props_move_copy, &msg);
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
    app_handle.listen_global(ChannelList::AXEventApp.to_string(), move |msg| {
        let axevent_app: AXEventApp = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        let widget_props = &mut *(match widget_props_move_copy.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        });

        match &axevent_app {
            AXEventApp::AppWindowFocused(msg) => {
                widget_props.currently_focused_app_window = Some(msg.window);
            }
            AXEventApp::AppWindowMoved(msg) => {
                on_move_app_window(widget_props, &msg);
            }
            AXEventApp::AppUIElementFocused(_) => {}
            AXEventApp::AppActivated(msg) => {
                widget_props.is_app_focused = true;
                widget_props.currently_focused_app_window = msg.focused_app_window;
            }
            AXEventApp::AppDeactivated(msg) => {
                on_deactivate_app(widget_props, &msg);
            }
            AXEventApp::AppContentActivationChange(msg) => {
                on_toggle_content_window(widget_props, msg);
            }
        }

        // Checking if the widget window is focused and if it is, it starts dragging it.
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
