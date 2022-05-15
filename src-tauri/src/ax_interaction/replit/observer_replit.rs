use std::thread;

use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification, kAXMainWindowChangedNotification,
    kAXValueChangedNotification, kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification, kAXWindowMovedNotification, kAXWindowResizedNotification,
};

use accessibility::{AXObserver, AXUIElement, Error};
use core_foundation::runloop::CFRunLoop;

use super::callback_replit_notifications;
use crate::ax_interaction::{
    models::editor::EditorAppClosedMessage, AXEventReplit, ReplitObserverState,
};

static CHROME_BUNDLE_ID: &str = "com.google.Chrome";
static OBSERVER_REGISTRATION_DELAY_IN_MILLIS: u64 = 2000;

static OBSERVER_NOTIFICATIONS: &'static [&'static str] = &[
    kAXApplicationActivatedNotification,
    kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification,
    kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification,
    kAXMainWindowChangedNotification,
    kAXValueChangedNotification,
    kAXWindowCreatedNotification,
    kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification,
    kAXWindowMovedNotification,
    kAXWindowResizedNotification,
];

pub fn register_observer_replit(
    known_replit_app: &mut Option<AXUIElement>,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    // Register Replit observer
    // =======================
    // Happens only once when replit is launched; is retriggered if replit is restarted
    let registration_required_res = is_new_replit_observer_registration_required(known_replit_app);
    if let Ok(registration_required) = registration_required_res {
        if registration_required {
            if let Some(ref replit_app) = known_replit_app {
                create_observer_and_add_notifications(replit_app, &app_handle)?;
            }
        }
    } else {
        // Case: Replit is not running
        if let Some(ref replit_app) = known_replit_app {
            // Case: Replit was just closed

            AXEventReplit::EditorAppClosed(EditorAppClosedMessage {
                editor_name: "Replit".to_string(),
                pid: replit_app.pid().unwrap().try_into().unwrap(),
            })
            .publish_to_tauri(&app_handle);

            *known_replit_app = None;
        }
    }
    Ok(())
}

// Determine if a new observer is required. This might be the case if Replit was restarted. We detect this by
// checking if the Replit's AXUIElement has changed.
fn is_new_replit_observer_registration_required(
    known_replit_app: &mut Option<AXUIElement>,
) -> Result<bool, Error> {
    let replit_ui_element = AXUIElement::application_with_bundle(CHROME_BUNDLE_ID)?;

    if let Some(replit_app) = &known_replit_app {
        // Case: Replit has a new AXUIElement, telling us it was restarted
        if replit_ui_element != *replit_app {
            *known_replit_app = Some(replit_ui_element);
            return Ok(true);
        }
    } else {
        // Case: Replit was never started while the program was running; it's UI element is not known yet
        *known_replit_app = Some(replit_ui_element);
        return Ok(true);
    }

    Ok(false)
}

// This function is called to create a new observer and add the notifications to it.
// The list of notifications is managed at the top of the file in a static variable.
fn create_observer_and_add_notifications(
    replit_ui_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let pid = replit_ui_element.pid().unwrap();
    let app_handle_move_copy = app_handle.clone();
    thread::spawn(move || {
        // 0. Delay observer registration on macOS, because there is a good chance no
        // notifications will be received despite seemingly successful observer registration
        thread::sleep(std::time::Duration::from_millis(
            OBSERVER_REGISTRATION_DELAY_IN_MILLIS,
        ));

        // 1. Create AXObserver
        let replit_observer = AXObserver::new(pid, callback_replit_notifications);
        let ui_element = AXUIElement::application(pid);

        if let Ok(mut replit_observer) = replit_observer {
            // 2. Start AXObserver before adding notifications
            replit_observer.start();

            // 3. Add notifications
            for notification in OBSERVER_NOTIFICATIONS.iter() {
                let _ = replit_observer.add_notification(
                    notification,
                    &ui_element,
                    ReplitObserverState {
                        app_handle: app_handle_move_copy.clone(),
                        window_list: Vec::new(),
                    },
                );
            }

            // 4. Kick of RunLoop on this thread
            CFRunLoop::run_current();
        }
    });

    Ok(())
}
