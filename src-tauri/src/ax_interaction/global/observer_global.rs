use crate::ax_events_deprecated::models::{AppFocusState, AppInfo};
use crate::ax_events_deprecated::Event;
use crate::ax_interaction::XCodeObserverState;
use accessibility::{AXAttribute, AXUIElement, Error};

pub fn _observer_global(
    _ocused_app: &mut Option<AXUIElement>,
    _tauri_state: &XCodeObserverState,
) -> Result<(), Error> {
    // // Determine if user app focus has changed
    // // =======================================
    // let currently_focused_app = currently_focused_app()?;
    // if let Some(ref previously_focused_app) = focused_app {
    //     if (*previously_focused_app).pid()? != currently_focused_app.pid()? {
    //         // If focused UI element is xcode editor, emit editor_focus_change event as well

    //         // User app focus has changed
    //         let _ = _callback_global_app_focus(
    //             previously_focused_app,
    //             &currently_focused_app,
    //             &tauri_state,
    //         );
    //         *focused_app = Some(currently_focused_app);
    //     }
    // } else {
    //     // Case: first app in focus after program startup
    //     // User app focus has changed
    //     let _ = _callback_global_app_focus(
    //         &currently_focused_app,
    //         &currently_focused_app,
    //         &tauri_state,
    //     );

    //     *focused_app = Some(currently_focused_app);
    // }

    Ok(())
}

// This function emits an event when the user app focus changes.
// In order to allow other program logic to react, we need to pass the previous and current app to compare them against each other.
fn _callback_global_app_focus(
    previous_app: &AXUIElement,
    current_app: &AXUIElement,
    tauri_state: &XCodeObserverState,
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
    event.publish_to_tauri(tauri_state.app_handle.clone());

    Ok(())
}
