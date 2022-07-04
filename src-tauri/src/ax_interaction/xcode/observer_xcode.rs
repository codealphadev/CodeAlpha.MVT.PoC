use std::thread;

use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification, kAXMainWindowChangedNotification,
    kAXMenuItemSelectedNotification, kAXValueChangedNotification, kAXWindowCreatedNotification,
    kAXWindowDeminiaturizedNotification, kAXWindowMiniaturizedNotification,
    kAXWindowMovedNotification, kAXWindowResizedNotification,
};

use accessibility::{AXObserver, AXUIElement, Error};
use core_foundation::runloop::CFRunLoop;

use super::callback_xcode_notifications;
use crate::ax_interaction::{
    models::editor::EditorAppClosedMessage, AXEventXcode, XCodeObserverState,
};

static EDITOR_XCODE_BUNDLE_ID: &str = "com.apple.dt.Xcode";
static OBSERVER_REGISTRATION_DELAY_IN_MILLIS: u64 = 500;

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
    kAXMenuItemSelectedNotification,
];

pub fn register_observer_xcode(
    known_xcode_app: &mut Option<AXUIElement>,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    // Register XCode observer
    // =======================
    // Happens only once when xcode is launched; is retriggered if xcode is restarted
    let registration_required_res = is_new_xcode_observer_registration_required(known_xcode_app);
    if let Ok(registration_required) = registration_required_res {
        if registration_required {
            if let Some(ref xcode_app) = known_xcode_app {
                create_observer_and_add_notifications(xcode_app, &app_handle)?;
            }
        }
    } else {
        // Case: XCode is not running
        if let Some(ref xcode_app) = known_xcode_app {
            // Case: XCode was just closed

            AXEventXcode::EditorAppClosed(EditorAppClosedMessage {
                editor_name: "Xcode".to_string(),
                pid: xcode_app.pid().unwrap().try_into().unwrap(),
                browser: None,
            })
            .publish_to_tauri(&app_handle);

            *known_xcode_app = None;
        }
    }
    Ok(())
}

// Determine if a new observer is required. This might be the case if XCode was restarted. We detect this by
// checking if the XCode's AXUIElement has changed.
fn is_new_xcode_observer_registration_required(
    known_xcode_app: &mut Option<AXUIElement>,
) -> Result<bool, Error> {
    let xcode_ui_element = AXUIElement::application_with_bundle(EDITOR_XCODE_BUNDLE_ID)?;

    if let Some(xcode_app) = &known_xcode_app {
        // Case: XCode has a new AXUIElement, telling us it was restarted
        if xcode_ui_element != *xcode_app {
            *known_xcode_app = Some(xcode_ui_element);
            return Ok(true);
        }
    } else {
        // Case: XCode was never started while the program was running; it's UI element is not known yet
        *known_xcode_app = Some(xcode_ui_element);
        return Ok(true);
    }

    Ok(false)
}

// This function is called to create a new observer and add the notifications to it.
// The list of notifications is managed at the top of the file in a static variable.
fn create_observer_and_add_notifications(
    xcode_ui_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let pid = xcode_ui_element.pid().unwrap();
    let app_handle_move_copy = app_handle.clone();
    thread::spawn(move || {
        // 0. Delay observer registration on macOS, because there is a good chance no
        // notifications will be received despite seemingly successful observer registration
        thread::sleep(std::time::Duration::from_millis(
            OBSERVER_REGISTRATION_DELAY_IN_MILLIS,
        ));

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
                    XCodeObserverState {
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
