#![allow(non_upper_case_globals)]

use std::{ffi::c_void, mem};

use accessibility::{AXAttribute, AXObserver, AXUIElement};
use accessibility_sys::{AXObserverRef, AXUIElementRef};
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};

use crate::ax_interaction::utils::TauriState;

use colored::*;

// This file contains the callback function that is registered with the AXObserver
// that listens to notifications on our own app.
//
// Adjacent files contain the control logic for the different notifications received

/// Entry callback function that is being called by the operating system every time
/// one of the registered notifications is received.
pub unsafe extern "C" fn callback_app_notifications(
    observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    context: *mut c_void,
) {
    let _observer: AXObserver = TCFType::wrap_under_get_rule(observer);
    let _element: AXUIElement = TCFType::wrap_under_get_rule(element);
    let notification = CFString::wrap_under_get_rule(notification);
    let _context: *mut TauriState = mem::transmute(context);

    let role = _element.attribute(&AXAttribute::role());
    let title = _element.attribute(&AXAttribute::title());
    println!(
        "{}, role: {}, role: {}",
        format!("{:?}", title).green(),
        notification.to_string().bold().red(),
        format!("{:?}", role).blue(),
    );

    match notification.to_string().as_str() {
        _other => {}
    }
}
