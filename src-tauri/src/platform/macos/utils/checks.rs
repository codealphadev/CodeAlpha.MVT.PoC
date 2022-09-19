use accessibility::{AXAttribute, AXUIElement};
use accessibility_sys::{
    kAXTrustedCheckOptionPrompt, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
};
use core_foundation::{
    base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
};

use crate::platform::macos::setup::{get_registered_ax_observer, ObserverType};

use super::{ax_helpers::ax_attribute, internal::get_focused_uielement, GetVia, XcodeError};

fn _is_focused_uielement_xcode_editor_textarea() -> bool {
    if let Ok(xcode_textarea) = get_xcode_editor_textarea() {
        xcode_textarea.is_some()
    } else {
        false
    }
}

pub fn get_xcode_editor_textarea() -> Result<Option<AXUIElement>, XcodeError> {
    if let Some((pid, _)) = get_registered_ax_observer(ObserverType::XCode) {
        let uielement = get_focused_uielement(&GetVia::Pid(pid))?;

        if is_uielement_xcode_editor_textarea(&uielement)? {
            return Ok(Some(uielement));
        } else {
            return Ok(None);
        }
    } else {
        Err(XcodeError::WindowHashUnknown)
    }
}

pub fn is_uielement_xcode_editor_textarea(uielement: &AXUIElement) -> Result<bool, XcodeError> {
    if ax_attribute(uielement, AXAttribute::role())? == "AXTextArea"
        && ax_attribute(uielement, AXAttribute::description())? == "Source Editor"
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn is_currently_focused_app_our_app() -> Result<bool, XcodeError> {
    let focused_uielement = get_focused_uielement(&GetVia::Current)?;

    if let Some((app_pid, _)) = get_registered_ax_observer(ObserverType::App) {
        match focused_uielement.pid() {
            Ok(pid) => {
                if pid == app_pid {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(ax_error) => Err(XcodeError::AXError(ax_error.into())),
        }
    } else {
        Err(XcodeError::WindowHashUnknown)
    }
}

/// Checks whether or not this application is a trusted accessibility client.
pub fn is_application_trusted() -> bool {
    unsafe {
        return AXIsProcessTrusted();
    }
}

/// Same as [is_application_trusted], but also shows the user a prompt asking
/// them to allow accessibility API access if it hasn't already been given.
pub fn is_application_trusted_with_prompt() -> bool {
    unsafe {
        let option_prompt = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
        let dict: CFDictionary<CFString, CFBoolean> =
            CFDictionary::from_CFType_pairs(&[(option_prompt, CFBoolean::true_value())]);
        return AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef());
    }
}
