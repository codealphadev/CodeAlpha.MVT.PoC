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
use crate::{
    app_handle,
    ax_interaction::{
        models::editor::EditorAppClosedMessage,
        setup::{
            get_registered_ax_observer, remove_registered_ax_observer,
            store_registered_ax_observer, ObserverType,
        },
        AXEventXcode, XCodeObserverState,
    },
};

static EDITOR_XCODE_BUNDLE_ID: &str = "com.apple.dt.Xcode";

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

pub fn register_observer_xcode() -> Result<(), Error> {
    if is_new_xcode_observer_registration_required()? {
        // Cleanup old observers
        if let Some((_, observer)) = remove_registered_ax_observer(ObserverType::XCode) {
            observer.stop();
        }

        // Create new observer
        std::thread::spawn(|| {
            _ = create_observer_and_add_notifications();
        });
    }

    Ok(())
}

// Determine if a new observer is required. This might be the case if XCode was restarted. We detect this by
// checking if the XCode's AXUIElement has changed.
fn is_new_xcode_observer_registration_required() -> Result<bool, Error> {
    // Determine pid of xcode
    let xcode_pid = AXUIElement::application_with_bundle(EDITOR_XCODE_BUNDLE_ID)?.pid()?;

    if let Some((pid, _)) = get_registered_ax_observer(ObserverType::XCode) {
        if pid == xcode_pid {
            // Case: The registered observer for Xcode has the correct pid.
            return Ok(false);
        } else {
            // Case: XCode was just closed
            AXEventXcode::EditorAppClosed(EditorAppClosedMessage {
                editor_name: "Xcode".to_string(),
                pid: pid.try_into().unwrap(),
                browser: None,
            })
            .publish_to_tauri(&app_handle());
        }
    }

    Ok(true)
}

// This function is called to create a new observer and add the notifications to it.
// The list of notifications is managed at the top of the file in a static variable.
fn create_observer_and_add_notifications() -> Result<(), Error> {
    let xcode_pid = AXUIElement::application_with_bundle(EDITOR_XCODE_BUNDLE_ID)?.pid()?;

    // 1. Create AXObserver
    let mut xcode_observer = AXObserver::new(xcode_pid, callback_xcode_notifications)?;
    let ui_element = AXUIElement::application(xcode_pid);

    store_registered_ax_observer(xcode_pid, ObserverType::XCode, &xcode_observer);

    // 2. Start AXObserver before adding notifications
    xcode_observer.start();

    // 3. Add notifications
    for notification in OBSERVER_NOTIFICATIONS.iter() {
        let _ = xcode_observer.add_notification(
            notification,
            &ui_element,
            XCodeObserverState {
                app_handle: app_handle(),
                window_list: Vec::new(),
            },
        );
    }

    // 4. Kick of RunLoop on this thread
    CFRunLoop::run_current();

    Ok(())
}
