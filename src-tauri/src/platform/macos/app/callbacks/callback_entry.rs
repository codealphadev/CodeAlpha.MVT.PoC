#![allow(non_upper_case_globals)]

use std::{ffi::c_void, mem};

use accessibility::{AXObserver, AXUIElement};
use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification, kAXMainWindowChangedNotification,
    kAXWindowMovedNotification, AXObserverRef, AXUIElementRef,
};
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};

use crate::platform::macos::app::AppObserverState;

use super::{
    notifiy_app_activated, notifiy_app_deactivated, notify_uielement_focused,
    notify_window_focused, notify_window_moved,
};

// This file contains the callback function that is registered with the AXObserver
// that listens to notifications on our own app.
//
// Adjacent files contain the control logic for the different notifications received
pub unsafe extern "C" fn callback_app_notifications(
    observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    context: *mut c_void,
) {
    let _observer: AXObserver = TCFType::wrap_under_get_rule(observer);
    let element: AXUIElement = TCFType::wrap_under_get_rule(element);
    let notification = CFString::wrap_under_get_rule(notification);
    let context: *mut AppObserverState = mem::transmute(context);

    match notification.to_string().as_str() {
        kAXFocusedUIElementChangedNotification => {
            let _ = notify_uielement_focused(&element, &mut (*context));
        }
        kAXMainWindowChangedNotification => {
            let _ = notify_window_focused(&element, &mut (*context));
        }
        kAXApplicationActivatedNotification => {
            let _ = notifiy_app_activated(&element, &mut (*context));
        }
        kAXApplicationDeactivatedNotification => {
            let _ = notifiy_app_deactivated(&element, &mut (*context));
        }
        kAXApplicationHiddenNotification => {
            let _ = notifiy_app_deactivated(&element, &mut (*context));
        }
        kAXApplicationShownNotification => {
            let _ = notifiy_app_activated(&element, &mut (*context));
        }
        kAXWindowMovedNotification => {
            let _ = notify_window_moved(&element, &mut (*context));
        }
        _other => {
            println!("Forgotten notification: {:?}", _other)
        }
    }
}
