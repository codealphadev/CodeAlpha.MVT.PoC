#![allow(non_upper_case_globals)]

use std::{ffi::c_void, mem};

use accessibility::{AXObserver, AXUIElement, AXUIElementAttributes};
use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification, kAXMainWindowChangedNotification,
    kAXMenuItemSelectedNotification, kAXValueChangedNotification, kAXWindowCreatedNotification,
    kAXWindowDeminiaturizedNotification, kAXWindowMiniaturizedNotification,
    kAXWindowMovedNotification, kAXWindowResizedNotification, AXObserverRef, AXUIElementRef,
};
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};

use crate::platform::macos::xcode::{
    callbacks::{notify_window_created, notify_window_destroyed},
    XCodeObserverState,
};

use super::{
    notification_key_press_save, notify_app_activated, notify_app_deactivated,
    notify_uielement_focused, notify_value_changed, notify_window_moved, notify_window_resized,
};

// This file contains the callback function that is registered with the AXObserver
// that listens to notifications on the XCode AXUIElement.
//
// Adjacent files contain the control logic for the different notifications received
pub unsafe extern "C" fn callback_xcode_notifications(
    observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    context: *mut c_void,
) {
    let _observer: AXObserver = TCFType::wrap_under_get_rule(observer);
    let element: AXUIElement = TCFType::wrap_under_get_rule(element);
    let notification = CFString::wrap_under_get_rule(notification);
    let context: *mut XCodeObserverState = mem::transmute(context);

    // In case the window that contains the ui element which triggered this notification is not
    // yet contained in the window_list of XCodeObserverState, we trigger notify_window_created() msg first.
    let xcode_observer_state = &(*context);
    if xcode_observer_state.window_list.len() == 0 {
        if let Ok(window_elem) = element.window() {
            let _ = notify_window_created(&window_elem, &mut (*context));
        }
    }

    // This UI element is not of interest; we ignore it right here.
    if let Ok(role) = element.role() {
        if role == "AXImage" {
            return;
        }
    }

    match notification.to_string().as_str() {
        kAXFocusedUIElementChangedNotification => {
            let _ = notify_uielement_focused(&element, &mut (*context));
        }
        kAXValueChangedNotification => {
            let _ = notify_value_changed(&element, &mut (*context));
        }
        kAXMainWindowChangedNotification => {
            let _ = notify_window_created(&element, &mut (*context));
            let _ = notify_window_destroyed(&element, &mut (*context));
        }
        kAXWindowCreatedNotification => {
            let _ = notify_window_created(&element, &mut (*context));
        }
        kAXApplicationActivatedNotification => {
            let _ = notify_window_created(&element, &mut (*context));
            let _ = notify_window_destroyed(&element, &mut (*context));
            let _ = notify_app_activated(&element, &mut (*context));
        }
        kAXApplicationDeactivatedNotification => {
            let _ = notify_app_deactivated(&element, &mut (*context));
            let _ = notify_window_destroyed(&element, &mut (*context));
        }
        kAXApplicationHiddenNotification => {
            let _ = notify_app_deactivated(&element, &mut (*context));
        }
        kAXApplicationShownNotification => {
            let _ = notify_app_activated(&element, &mut (*context));
        }
        kAXWindowMovedNotification => {
            let _ = notify_window_moved(&element, &mut (*context));
        }
        kAXWindowResizedNotification => {
            let _ = notify_window_resized(&element, &mut (*context));
        }
        kAXWindowMiniaturizedNotification => {
            // Here we do nothing, because this behavior would be duplicated with kAXFocusedUIElementChangedNotification
        }
        kAXWindowDeminiaturizedNotification => {
            // Here we do nothing, because this behavior would be duplicated with kAXFocusedUIElementChangedNotification
        }
        kAXMenuItemSelectedNotification => {
            let _ = notification_key_press_save(&element, &mut (*context));
        }
        _other => {
            println!("Forgotten notification: {:?}", _other)
        }
    }
}
