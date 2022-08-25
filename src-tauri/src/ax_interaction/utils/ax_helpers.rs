use accessibility::{AXAttribute, AXUIElement, Error};
use core_foundation::base::{CFHash, TCFType};
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum GetVia {
    Current,
    Hash(usize),
    Pid(i32),
    UIElem(AXUIElement),
}

#[derive(Error, Debug)]
pub enum XcodeError {
    #[error("Calling macOS AX API failed: {0}.")]
    AXCallFailed(i32),
    #[error("AX resource not found.")]
    AXResourceNotFound,
    #[error("The focused window is not an Xcode editor window.")]
    FocusedWindowNotXcode,
    #[error("The focused UI element not an editor textarea.")]
    FocusedUIElemNotTextarea,
    #[error("No open Xcode editor windows found with this hash.")]
    WindowHashUnknown,
}

impl XcodeError {
    pub fn map_ax_error(ax_error: Error) -> XcodeError {
        match ax_error {
            Error::NotFound => XcodeError::AXResourceNotFound,
            Error::Ax(id) => XcodeError::AXCallFailed(id),
        }
    }
}

pub fn generate_axui_element_hash(ui_element: &AXUIElement) -> usize {
    unsafe { CFHash(ui_element.as_CFTypeRef()) }
}

pub fn ax_attribute<T: TCFType>(
    ui_elem: &AXUIElement,
    attribute: AXAttribute<T>,
) -> Result<T, XcodeError> {
    ui_elem
        .attribute(&attribute)
        .map_err(|err| XcodeError::map_ax_error(err))
}

pub fn set_ax_attribute<T: TCFType>(
    ui_elem: &AXUIElement,
    attribute: AXAttribute<T>,
    value: impl Into<T>,
) -> Result<(), XcodeError> {
    ui_elem
        .set_attribute(&attribute, value)
        .map_err(|err| XcodeError::map_ax_error(err))
}
