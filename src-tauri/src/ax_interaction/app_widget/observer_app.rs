use std::thread;

use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification, kAXFocusedWindowChangedNotification,
    kAXMainWindowChangedNotification, kAXMovedNotification, kAXResizedNotification,
    kAXSelectedChildrenChangedNotification, kAXSelectedChildrenMovedNotification,
    kAXUIElementDestroyedNotification, kAXWindowCreatedNotification,
    kAXWindowDeminiaturizedNotification, kAXWindowMiniaturizedNotification,
};

use accessibility::{AXObserver, AXUIElement, Error};
use core_foundation::runloop::CFRunLoop;

use super::callback_app_widget;
use crate::ax_interaction::utils::TauriState;

static OBSERVER_NOTIFICATIONS: &'static [&'static str] = &[
    kAXFocusedUIElementChangedNotification,
    kAXFocusedWindowChangedNotification,
    kAXApplicationShownNotification,
    kAXApplicationHiddenNotification,
    kAXWindowCreatedNotification,
    kAXMainWindowChangedNotification,
    kAXApplicationDeactivatedNotification,
    kAXApplicationActivatedNotification,
    kAXWindowMiniaturizedNotification,
    kAXWindowDeminiaturizedNotification,
    kAXUIElementDestroyedNotification,
    kAXSelectedChildrenMovedNotification,
    kAXSelectedChildrenChangedNotification,
    kAXResizedNotification,
    kAXMovedNotification,
];

/// AX Observer - Our App
/// ================================
/// This call registers a macOS AXObserver for our application
/// The list of notifications added to this observer can be modified at the
/// top of the file in a static array.
pub fn observer_app(handle: &tauri::AppHandle) -> Result<(), Error> {
    create_observer_and_add_notifications(&TauriState {
        handle: handle.clone(),
    })?;
    Ok(())
}

/// This function is called to create a new observer and add the notifications to it.
/// The list of notifications is managed at the top of the file in a static variable.
fn create_observer_and_add_notifications(tauri_apphandle: &TauriState) -> Result<(), Error> {
    let tauri_handle_move_copy = tauri_apphandle.handle.clone();
    thread::spawn(move || {
        // let pid: i32 = std::process::id().try_into().unwrap();
        let pid: i32 = 48049;

        // 1. Create AXObserver
        let app_observer = AXObserver::new(pid, callback_app_widget);
        let ui_element = AXUIElement::application(pid);

        if let Ok(mut app_observer) = app_observer {
            // 2. Start AXObserver before adding notifications
            app_observer.start();

            // 3. Add notifications
            for notification in OBSERVER_NOTIFICATIONS.iter() {
                let _ = app_observer.add_notification(
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
