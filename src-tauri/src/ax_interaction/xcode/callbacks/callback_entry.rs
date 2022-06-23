#![allow(non_upper_case_globals)]

use std::{ffi::c_void, mem};

use accessibility::{AXObserver, AXUIElement, AXUIElementAttributes};
use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification, kAXMainWindowChangedNotification,
    kAXValueChangedNotification, kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification, kAXWindowMovedNotification, kAXWindowResizedNotification,
    AXObserverRef, AXUIElementRef,
};
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};

use crate::ax_interaction::{
    generate_axui_element_hash,
    xcode::callbacks::{notify_window_created, notify_window_destroyed},
    XCodeObserverState,
};

use super::{
    notifiy_app_activated, notifiy_app_deactivated, notify_uielement_focused, notify_value_changed,
    notify_window_moved, notify_window_resized,
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
            let _ = notifiy_app_activated(&element, &mut (*context));
        }
        kAXApplicationDeactivatedNotification => {
            let _ = notifiy_app_deactivated(&element, &mut (*context));
            let _ = notify_window_destroyed(&element, &mut (*context));
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
        kAXWindowResizedNotification => {
            let _ = notify_window_resized(&element, &mut (*context));
        }
        kAXWindowMiniaturizedNotification => {
            // Here we do nothing, because this behavior would be duplicated with kAXFocusedUIElementChangedNotification
        }
        kAXWindowDeminiaturizedNotification => {
            // Here we do nothing, because this behavior would be duplicated with kAXFocusedUIElementChangedNotification
        }
        _other => {
            println!("Forgotten notification: {:?}", _other)
        }
    }
}

/// It prints out all the windows of the application that owns the active window, and highlights the
/// active window
///
/// Arguments:
///
/// * `elem`: The element that was updated.
/// * `_state`: The state of the observer.
fn _plot_active_window(elem: &AXUIElement, _state: &XCodeObserverState) {
    if let Ok(window_elem) = elem.window() {
        if let Ok(application) = window_elem.parent() {
            if let Ok(windows) = application.windows() {
                println!("==========================");
                println!("All windows:");

                let window_hash = generate_axui_element_hash(&window_elem);

                let mut found_hash = false;
                for elem in &windows {
                    let elem_hash = generate_axui_element_hash(&elem);
                    if elem_hash == window_hash {
                        println!("-> Hash {}", elem_hash);
                        found_hash = true;
                    } else {
                        println!("Hash {}", elem_hash);
                    }
                }

                if !found_hash {
                    println!("No window with hash {} found", window_hash);
                }
            }
        }
    }
}
