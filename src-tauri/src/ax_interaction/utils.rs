use accessibility::{AXAttribute, AXUIElement, Error};
use accessibility_sys::{
    kAXTrustedCheckOptionPrompt, pid_t, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
};
use core_foundation::{
    base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
};

static EDITOR_NAME: &str = "Xcode";

// Method to get the focused AXUIElement's top-level window
pub fn currently_focused_app() -> Result<AXUIElement, Error> {
    let system_wide_element = AXUIElement::system_wide();
    let focused_ui_element = system_wide_element.attribute(&AXAttribute::focused_uielement())?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    if let Ok(parent) = focused_window.attribute(&AXAttribute::parent()) {
        return Ok(parent);
    } else {
        return Ok(focused_ui_element);
    }
}

pub fn focused_uielement_of_app(app_pid: pid_t) -> Result<AXUIElement, Error> {
    let application = AXUIElement::application(app_pid);
    let focused_ui_element = application.attribute(&AXAttribute::focused_uielement())?;

    Ok(focused_ui_element)
}

pub fn is_focused_uielement_of_app_xcode_editor_field(app_pid: pid_t) -> Result<bool, Error> {
    let focused_ui_element = focused_uielement_of_app(app_pid)?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    let parent = focused_window.attribute(&AXAttribute::parent())?;
    let title = parent.attribute(&AXAttribute::title())?;

    let role = focused_ui_element.attribute(&AXAttribute::role())?;

    if role == "AXTextArea" && title == EDITOR_NAME {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Checks whether or not this application is a trusted accessibility client.
pub fn _application_is_trusted() -> bool {
    unsafe {
        return AXIsProcessTrusted();
    }
}

/// Same as [application_is_trusted], but also shows the user a prompt asking
/// them to allow accessibility API access if it hasn't already been given.
pub fn application_is_trusted_with_prompt() -> bool {
    unsafe {
        let option_prompt = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
        let dict: CFDictionary<CFString, CFBoolean> =
            CFDictionary::from_CFType_pairs(&[(option_prompt, CFBoolean::true_value())]);
        return AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef());
    }
}

// A bit WIP - tiny struct to help move context info into callback functions of the observers
pub struct TauriState {
    pub handle: tauri::AppHandle,
}
