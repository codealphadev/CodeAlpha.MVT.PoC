use crate::ax_events::models::{AppFocusState, AppInfo};
use crate::ax_events::Event;
use crate::ax_interaction::utils::TauriState;
use crate::ax_interaction::xcode::notification_focused_uielement;
use crate::ax_interaction::{
    currently_focused_app, focused_uielement_of_app, is_focused_uielement_of_app_xcode_editor_field,
};
use accessibility::{AXAttribute, AXUIElement, Error};

pub fn observer_global(
    focused_app: &mut Option<AXUIElement>,
    tauri_state: &TauriState,
) -> Result<(), Error> {
    // Determine if user app focus has changed
    // =======================================
    let currently_focused_app = currently_focused_app()?;
    if let Some(ref previously_focused_app) = focused_app {
        if (*previously_focused_app).pid()? != currently_focused_app.pid()? {
            // If focused UI element is xcode editor, emit editor_focus_change event as well
            let _ = callback_xcode_editor_focus(&currently_focused_app, &tauri_state);

            // User app focus has changed
            let _ = callback_global_app_focus(
                previously_focused_app,
                &currently_focused_app,
                &tauri_state,
            );
            *focused_app = Some(currently_focused_app);
        }
    } else {
        // If focused UI element is xcode editor, emit editor_focus_change event as well
        let _ = callback_xcode_editor_focus(&currently_focused_app, &tauri_state);

        // Case: first app in focus after program startup
        // User app focus has changed
        let _ =
            callback_global_app_focus(&currently_focused_app, &currently_focused_app, &tauri_state);

        *focused_app = Some(currently_focused_app);
    }

    Ok(())
}

// This function emits an event when the user app focus changes.
// In order to allow other program logic to react, we need to pass the previous and current app to compare them against each other.
fn callback_global_app_focus(
    previous_app: &AXUIElement,
    current_app: &AXUIElement,
    tauri_state: &TauriState,
) -> Result<(), Error> {
    // Remark: The first app that is being focused after startup emits this event with previous_app and current_app equal.
    let current_app_title = current_app.attribute(&AXAttribute::title())?;
    let previous_app_title = previous_app.attribute(&AXAttribute::title())?;

    let focus_state = AppFocusState {
        previous_app: AppInfo {
            name: previous_app_title.to_string(),
            pid: previous_app.pid()?.try_into().unwrap(),
        },
        current_app: AppInfo {
            name: current_app_title.to_string(),
            pid: current_app.pid()?.try_into().unwrap(),
        },
    };

    let event = Event::AppFocusState(focus_state);
    event.publish_to_tauri(tauri_state.handle.clone());

    Ok(())
}

fn callback_xcode_editor_focus(
    currently_focused_app: &AXUIElement,
    tauri_state: &TauriState,
) -> Result<(), Error> {
    if let Ok(pid) = currently_focused_app.pid() {
        if is_focused_uielement_of_app_xcode_editor_field(pid)? {
            return notification_focused_uielement(&focused_uielement_of_app(pid)?, &tauri_state);
        }
    }

    Ok(())
}
