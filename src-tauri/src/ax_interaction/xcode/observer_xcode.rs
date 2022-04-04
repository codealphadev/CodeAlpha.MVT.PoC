use std::thread;

use accessibility_sys::{
    kAXFocusedUIElementChangedNotification, kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification, kAXWindowMovedNotification, kAXWindowResizedNotification,
};

use accessibility::{AXAttribute, AXObserver, AXUIElement, Error};
use core_foundation::runloop::CFRunLoop;

use super::callback_xcode_notifications;
use crate::ax_interaction::{currently_focused_app, utils::TauriState};

static EDITOR_NAME: &str = "Xcode";
static OBSERVER_NOTIFICATIONS: &'static [&'static str] = &[
    kAXFocusedUIElementChangedNotification,
    kAXWindowMovedNotification,
    kAXWindowResizedNotification,
    kAXWindowMiniaturizedNotification,
    kAXWindowDeminiaturizedNotification,
];

pub fn observer_xcode(
    xcode_app: &mut Option<AXUIElement>,
    tauri_state: &TauriState,
) -> Result<(), Error> {
    // Register XCode observer
    // =======================
    // Happens only once when xcode is launched; is retriggered if xcode is restarted
    let app = currently_focused_app();
    if let Ok(currently_focused_app) = app {
        let registration_required_res =
            is_new_xcode_observer_registration_required(currently_focused_app, xcode_app);

        if let Ok(registration_required) = registration_required_res {
            if registration_required {
                if let Some(ref xcode_app) = xcode_app {
                    let _ = create_observer_and_add_notifications(xcode_app, &tauri_state);
                }
            }
        }
    }

    Ok(())
}

// Determine if a new observer is required. This might be the case if XCode was restarted. We detect this by
// checking if the XCode's AXUIElement has changed.
fn is_new_xcode_observer_registration_required(
    focused_app: AXUIElement,
    known_xcode_app: &mut Option<AXUIElement>,
) -> Result<bool, Error> {
    let focused_app_title = focused_app.attribute(&AXAttribute::title())?;

    // Check if focused app is XCode, skip if not
    if focused_app_title != EDITOR_NAME {
        return Ok(false);
    }

    if let Some(xcode_app) = known_xcode_app {
        let xcode_app_identifier = xcode_app.attribute(&AXAttribute::identifier())?;
        let focused_app_identifier = focused_app.attribute(&AXAttribute::identifier())?;

        // Case: XCode has a new AXUIElement, telling us it was restarted
        if xcode_app_identifier != focused_app_identifier {
            *known_xcode_app = Some(focused_app);
            return Ok(true);
        }
    } else {
        // Case: XCode was never started while the program was running; it's UI element is not known yet
        *known_xcode_app = Some(focused_app);
        return Ok(true);
    }

    Ok(false)
}

// This function is called to create a new observer and add the notifications to it.
// The list of notifications is managed at the top of the file in a static variable.
fn create_observer_and_add_notifications(
    xcode_ui_element: &AXUIElement,
    tauri_apphandle: &TauriState,
) -> Result<(), Error> {
    assert_eq!(
        xcode_ui_element.attribute(&AXAttribute::title())?,
        EDITOR_NAME
    );

    let pid = xcode_ui_element.pid().unwrap();
    let tauri_handle_move_copy = tauri_apphandle.handle.clone();
    thread::spawn(move || {
        // 1. Create AXObserver
        let xcode_observer = AXObserver::new(pid, callback_xcode_notifications);
        let ui_element = AXUIElement::application(pid);

        if let Ok(mut xcode_observer) = xcode_observer {
            // 2. Start AXObserver before adding notifications
            xcode_observer.start();

            // 3. Add notifications
            for notification in OBSERVER_NOTIFICATIONS.iter() {
                let _ = xcode_observer.add_notification(
                    notification,
                    &ui_element,
                    TauriState {
                        handle: tauri_handle_move_copy.clone(),
                    },
                );
            }

            // 4. Kick of RunLoop on this thread
            CFRunLoop::run_current();
        }
    });

    Ok(())
}
