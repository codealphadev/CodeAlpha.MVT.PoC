use accessibility::{AXAttribute, AXUIElement};
use core_foundation::base::{CFHash, TCFType};

#[derive(Clone, Debug)]
pub enum GetVia {
    Current,
    Hash(usize),
    Pid(i32),
    UIElem(AXUIElement),
}

#[derive(thiserror::Error, Debug)]
pub enum XcodeError {
    #[error("The focused window is not an Xcode editor window.")]
    FocusedWindowNotXcode,
    #[error("The focused UI element not an editor textarea.")]
    FocusedUIElemNotTextarea,
    #[error("No open Xcode editor windows found with this hash.")]
    WindowHashUnknown,
    #[error("Implausible dimensions computation, e.g. a result turning out negative when it's not supposed to be.")]
    ImplausibleDimensions,
    #[error("The computation for bounding rectangle failed due to the textrange not being fully contained within Xcode's visible text range.")]
    NotContainedVisibleTextRange,
    #[error("Could not get the text content of the textarea.")]
    GettingTextContentFailed,
    #[error("Could not find the UI element.")]
    UIElementNotFound,
    #[error("XcodeError: {0}.")]
    CustomError(String),
    #[error("Calling macOS AX API failed.")]
    AXError(#[source] anyhow::Error),
    #[error("Something failed.")]
    GenericError(#[source] anyhow::Error),
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
        .map_err(|err| XcodeError::AXError(err.into()))
}

pub fn set_ax_attribute<T: TCFType>(
    ui_elem: &AXUIElement,
    attribute: AXAttribute<T>,
    value: impl Into<T>,
) -> Result<(), XcodeError> {
    ui_elem
        .set_attribute(&attribute, value)
        .map_err(|e| XcodeError::AXError(e.into()))
}
