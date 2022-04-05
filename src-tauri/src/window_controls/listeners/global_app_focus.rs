use std::sync::{Arc, Mutex};

use tauri::{EventHandler, Manager};

use crate::{
    ax_events::{models::AppFocusState, Event},
    ax_interaction::is_focused_uielement_of_app_xcode_editor_field,
    window_controls::parse_into_ax_event_type,
};

static EDITOR_NAME: &str = "Xcode";
static APP_NAME: &str = "CodeAlpha";
static EVENT_NAME: &str = "StateEvent-AppFocusState";

pub fn listen_global_app_focus(
    handle: tauri::AppHandle,
    listener_app_focus_status: &mut Option<EventHandler>,
    preserve_content_visibility_was_visible: &Arc<Mutex<bool>>,
    last_focused_app: &Arc<Mutex<Option<String>>>,
) {
    // Registering listener for a change in Global App Focus
    // =====================================
    // 1. Copy Arcs to be moved into closure
    let is_content_visibility_preserved = preserve_content_visibility_was_visible.clone();
    let app_handle = handle.clone();
    let last_focused_app_copy = last_focused_app.clone();

    // 2. Create listener
    let listener = handle.listen_global(EVENT_NAME, move |msg| {
        match parse_into_ax_event_type(msg) {
            Event::AppFocusState(val) => {
                control_logic(&val, app_handle.clone());

                // Update last focused app
                let mut last_focused_app = last_focused_app_copy.lock().unwrap();
                *last_focused_app = Some(val.current_app.name);
            }
            _ => {}
        }
    });

    *listener_app_focus_status = Some(listener);
}

fn control_logic(focus_state: &AppFocusState, handle: tauri::AppHandle) {
    // For now, on this listener, only hide widget if neither widget or editor are

    // Hide widget + content if neither editor nor widget is in focus
    // if ![APP_NAME, EDITOR_NAME].contains(&focus_state.current_app.name.as_str()) {
    //     Self::hide_widget_preserve_content(
    //         app_handle.clone(),
    //         &preserve_content_visibility_was_visible_copy,
    //     );
    // }

    // Show widget if editor text area is in focus
    if focus_state.current_app.name == EDITOR_NAME {
        if let Ok(is_editor) = is_focused_uielement_of_app_xcode_editor_field(
            focus_state.current_app.pid.try_into().unwrap(),
        ) {
            if is_editor {
                // TODO
            }
        }
    }
}
