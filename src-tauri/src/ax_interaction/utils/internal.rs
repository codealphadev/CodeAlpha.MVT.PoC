use accessibility::{AXAttribute, AXUIElement, AXUIElementAttributes, Error};
use core_graphics::geometry::CGRect;

use crate::{
    ax_interaction::{
        setup::{get_registered_ax_observer, ObserverType},
        utils::ax_helpers::generate_axui_element_hash,
    },
    utils::geometry::LogicalFrame,
};

use super::{
    super::{GetVia, XcodeError},
    ax_helpers::ax_attribute,
};

pub fn get_uielement_frame(ui_element: &AXUIElement) -> Result<LogicalFrame, XcodeError> {
    // Get Size and Origin of AXScrollArea
    let uielement_frame_axval = ax_attribute(&ui_element, AXAttribute::frame())?;

    match uielement_frame_axval.get_value::<CGRect>() {
        Ok(frame) => Ok(LogicalFrame::from_CGRect(&frame)),
        Err(ax_error) => Err(XcodeError::AXError(ax_error.into())),
    }
}

pub fn get_focused_uielement(get_via: &GetVia) -> Result<AXUIElement, XcodeError> {
    match get_via {
        GetVia::Hash(hash) => {
            if let Some((pid, _)) = get_registered_ax_observer(ObserverType::XCode) {
                let application = AXUIElement::application(pid);
                match get_window_uielement(application, *hash) {
                    Ok(window) => get_focused_uielement(&GetVia::UIElem(window)),
                    Err(ax_error) => Err(XcodeError::AXError(ax_error.into())),
                }
            } else {
                Err(XcodeError::WindowHashUnknown)
            }
        }
        GetVia::Pid(pid) => {
            let application = AXUIElement::application(*pid);
            match application.focused_uielement() {
                Ok(ui_elem) => Ok(ui_elem),
                Err(ax_error) => Err(XcodeError::AXError(ax_error.into())),
            }
        }
        GetVia::UIElem(uielem) => match uielem.focused_uielement() {
            Ok(ui_elem) => Ok(ui_elem),
            Err(ax_error) => Err(XcodeError::AXError(ax_error.into())),
        },
        GetVia::Current => {
            let system_wide_element = AXUIElement::system_wide();

            match system_wide_element.focused_uielement() {
                Ok(ui_elem) => Ok(ui_elem),
                Err(ax_error) => Err(XcodeError::AXError(ax_error.into())),
            }
        }
    }
}

fn get_window_uielement(app_uielement: AXUIElement, hash: usize) -> Result<AXUIElement, Error> {
    for window in &app_uielement.windows()? {
        if generate_axui_element_hash(&window) == hash {
            return Ok(window.clone());
        }
    }

    Err(Error::NotFound)
}
