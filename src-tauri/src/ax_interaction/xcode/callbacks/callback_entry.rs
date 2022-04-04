#![allow(non_upper_case_globals)]

use std::{ffi::c_void, mem};

use accessibility::{AXObserver, AXUIElement};
use accessibility_sys::{kAXFocusedUIElementChangedNotification, AXObserverRef, AXUIElementRef};
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};

use crate::ax_interaction::utils::TauriState;

use super::notification_focused_uielement;

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
    let context: *mut TauriState = mem::transmute(context);

    match notification.to_string().as_str() {
        kAXFocusedUIElementChangedNotification => {
            let _ = notification_focused_uielement(&element, &*context);
        }
        _ => {
            // println!("{:?}", other)
        }
    }
}
