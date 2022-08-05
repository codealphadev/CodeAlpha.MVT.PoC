use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::{
    core_engine::events::{models::CoreActivationStatusMessage, EventUserInteraction},
    utils::messaging::ChannelList,
    window_controls::code_overlay::{hide_code_overlay, show_code_overlay},
};

use super::WidgetWindow;

pub fn register_listener_user_interactions(
    app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
) {
    let widget_props_move_copy = (widget_props).clone();
    app_handle.listen_global(ChannelList::EventUserInteractions.to_string(), move |msg| {
        let event_user_interaction: EventUserInteraction =
            serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match event_user_interaction {
            EventUserInteraction::CoreActivationStatus(msg) => {
                on_core_activation_status_update(&widget_props_move_copy, &msg);
            }
            _ => {}
        }
    });
}

fn on_core_activation_status_update(
    widget_props: &Arc<Mutex<WidgetWindow>>,
    activation_msg: &CoreActivationStatusMessage,
) {
    let widget_props = &mut *(match widget_props.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    widget_props.update_code_overlay_visible(&activation_msg.engine_active);

    let editor_windows = &mut *(match widget_props.editor_windows.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows.get_mut(&focused_editor_window_id) {
            if let Some(engine_active) = activation_msg.engine_active {
                if engine_active {
                    let _ = show_code_overlay(
                        &widget_props.app_handle,
                        editor_window.textarea_position,
                        editor_window.textarea_size,
                    );
                } else {
                    let _ = hide_code_overlay(&widget_props.app_handle);
                }
            }
        }
    }
}
