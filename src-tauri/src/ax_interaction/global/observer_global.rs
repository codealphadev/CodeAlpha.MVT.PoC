use crate::ax_events::models::{AppFocusState, AppInfo};
use crate::ax_events::Event;
use crate::ax_interaction::currently_focused_app;
use crate::ax_interaction::utils::TauriState;
use accessibility::{AXAttribute, AXUIElement, Error};

pub fn observer_global(
    focused_app: &mut Option<AXUIElement>,
    tauri_state: &TauriState,
) -> Result<(), Error> {
    // Determine if user app focus has changed
    // =======================================
    let currently_focused_app = currently_focused_app()?;
    if let Some(ref previously_focused_app) = focused_app {
        if *previously_focused_app != currently_focused_app {
            let _ = callback_global_app_focus(
                previously_focused_app,
                &currently_focused_app,
                &tauri_state,
            );
            *focused_app = Some(currently_focused_app);
        }
    } else {
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
    assert_ne!(previous_app, current_app);

    let current_app_title = current_app.attribute(&AXAttribute::title())?;
    let previous_app_title = previous_app.attribute(&AXAttribute::title())?;

    let focus_state = AppFocusState {
        previous_app: AppInfo {
            name: previous_app_title.to_string(),
            pid: previous_app.pid()?,
        },
        current_app: AppInfo {
            name: current_app_title.to_string(),
            pid: current_app.pid()?,
        },
    };

    let event = Event::AppFocusState(focus_state);
    event.publish_to_tauri(tauri_state.handle.clone());

    Ok(())
}
