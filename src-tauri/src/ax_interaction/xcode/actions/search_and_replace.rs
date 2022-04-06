use accessibility::{AXAttribute, Error};
use accessibility_sys::pid_t;
use core_foundation::{
    base::{CFType, TCFType},
    string::CFString,
};

use crate::ax_interaction::{
    focused_uielement_of_app, is_focused_uielement_of_app_xcode_editor_field,
};

pub fn get_xcode_editor_content(pid: pid_t) -> Result<Option<String>, Error> {
    if is_focused_uielement_of_app_xcode_editor_field(pid)? {
        let editor_element = focused_uielement_of_app(pid)?;

        let content = editor_element.attribute(&AXAttribute::value())?;
        let content_str = content.downcast::<CFString>();

        if let Some(cf_str) = content_str {
            return Ok(Some(cf_str.to_string()));
        }
    }

    Ok(None)
}

pub fn update_xcode_editor_content(pid: pid_t, content: &str) -> Result<bool, Error> {
    if is_focused_uielement_of_app_xcode_editor_field(pid)? {
        let editor_element = focused_uielement_of_app(pid)?;

        let content_cf_str: CFString = content.into();

        editor_element.set_attribute(&AXAttribute::value(), content_cf_str.as_CFType())?;

        return Ok(true);
    }

    Ok(false)
}
