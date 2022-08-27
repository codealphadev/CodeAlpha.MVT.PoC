use accessibility::{AXAttribute, AXUIElement};
use core_foundation::{base::TCFType, string::CFString};

use super::{
    ax_helpers::{ax_attribute, set_ax_attribute},
    checks::is_uielement_xcode_editor_textarea,
    internal::get_focused_uielement,
    GetVia, XcodeError,
};

pub fn get_textarea_uielement(get_via: &GetVia) -> Result<AXUIElement, XcodeError> {
    let focused_uielement = get_focused_uielement(get_via)?;

    if is_uielement_xcode_editor_textarea(&focused_uielement)? {
        Ok(focused_uielement)
    } else {
        Err(XcodeError::FocusedUIElemNotTextarea)
    }
}

pub fn get_textarea_content(get_via: &GetVia) -> Result<String, XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;

    let content = ax_attribute(&textarea_uielement, AXAttribute::value())?;
    let content_str = content.downcast::<CFString>();

    if let Some(cf_str) = content_str {
        Ok(cf_str.to_string())
    } else {
        Err(XcodeError::GettingTextContentFailed)
    }
}

pub fn set_textarea_content(content: &String, get_via: &GetVia) -> Result<(), XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;

    let content_cf_str: CFString = content.as_str().into();

    set_ax_attribute(
        &textarea_uielement,
        AXAttribute::value(),
        content_cf_str.as_CFType(),
    )
}

pub fn get_textarea_file_path(get_via: &GetVia) -> Result<String, XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;
    let window_uielement = ax_attribute(&textarea_uielement, AXAttribute::window())?;

    let full_file_path = ax_attribute(&window_uielement, AXAttribute::document())?.to_string();
    let (_, file_path) = full_file_path.split_at(7);

    Ok(file_path.to_string())
}
