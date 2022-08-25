use accessibility::{AXUIElement, AXUIElementAttributes, Error};

use crate::ax_interaction::{
    setup::{get_registered_ax_observer, ObserverType},
    utils::ax_helpers::generate_axui_element_hash,
};

use super::super::{GetVia, XcodeError};

pub fn get_focused_uielement(get_via: GetVia) -> Result<AXUIElement, XcodeError> {
    match get_via {
        GetVia::Hash(hash) => {
            if let Some((pid, _)) = get_registered_ax_observer(ObserverType::XCode) {
                let application = AXUIElement::application(pid);
                match get_window_uielement(application, hash) {
                    Ok(window) => get_focused_uielement(GetVia::UIElem(window)),
                    Err(ax_error) => Err(XcodeError::map_ax_error(ax_error)),
                }
            } else {
                Err(XcodeError::WindowHashUnknown)
            }
        }
        GetVia::Pid(pid) => {
            let application = AXUIElement::application(pid);
            match application.focused_uielement() {
                Ok(ui_elem) => Ok(ui_elem),
                Err(ax_error) => Err(XcodeError::map_ax_error(ax_error)),
            }
        }
        GetVia::UIElem(uielem) => match uielem.focused_uielement() {
            Ok(ui_elem) => Ok(ui_elem),
            Err(ax_error) => Err(XcodeError::map_ax_error(ax_error)),
        },
        GetVia::Current => {
            let system_wide_element = AXUIElement::system_wide();

            match system_wide_element.focused_uielement() {
                Ok(ui_elem) => Ok(ui_elem),
                Err(ax_error) => Err(XcodeError::map_ax_error(ax_error)),
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
