use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification, kAXMainWindowChangedNotification,
    kAXWindowMovedNotification,
};

use accessibility::{AXObserver, AXUIElement, Error};
use core_foundation::runloop::CFRunLoop;

use crate::{
    app_handle,
    ax_interaction::{
        setup::{store_registered_ax_observer, ObserverType},
        AppObserverState,
    },
};

use super::callback_app_notifications;

static OBSERVER_NOTIFICATIONS: &'static [&'static str] = &[
    kAXFocusedUIElementChangedNotification,
    kAXMainWindowChangedNotification,
    kAXApplicationActivatedNotification,
    kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification,
    kAXApplicationShownNotification,
    kAXWindowMovedNotification,
];

/// AX Observer - Our App
/// ================================
/// This call registers a macOS AXObserver for our application
/// The list of notifications added to this observer can be modified at the
/// top of the file in a static array.
pub fn register_observer_app() -> Result<(), Error> {
    create_observer_and_add_notifications()
}

/// This function is called to create a new observer and add the notifications to it.
/// The list of notifications is managed at the top of the file in a static variable.
fn create_observer_and_add_notifications() -> Result<(), Error> {
    let pid: i32 = std::process::id().try_into().unwrap();

    // 1. Create AXObserver
    let app_observer = AXObserver::new(pid, callback_app_notifications);
    let ui_element = AXUIElement::application(pid);

    if let Ok(mut app_observer) = app_observer {
        store_registered_ax_observer(pid, ObserverType::App, &app_observer);

        // 2. Start AXObserver before adding notifications
        app_observer.start();

        // 3. Add notifications
        for notification in OBSERVER_NOTIFICATIONS.iter() {
            let _ = app_observer.add_notification(
                notification,
                &ui_element,
                AppObserverState {
                    app_handle: app_handle(),
                },
            );
        }

        // 4. Kick of RunLoop on this thread
        CFRunLoop::run_current();
    }

    Ok(())
}
