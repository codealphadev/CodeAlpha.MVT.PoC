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

    if let Ok(uielement_frame) = uielement_frame_axval.get_value::<CGRect>() {
        Ok(LogicalFrame::from_CGRect(&uielement_frame))
    } else {
        Err(XcodeError::AXResourceNotFound)
    }
}

pub fn get_focused_uielement(get_via: &GetVia) -> Result<AXUIElement, XcodeError> {
    match get_via {
        GetVia::Hash(hash) => {
            if let Some((pid, _)) = get_registered_ax_observer(ObserverType::XCode) {
                let application = AXUIElement::application(pid);
                match get_window_uielement(application, *hash) {
                    Ok(window) => get_focused_uielement(&GetVia::UIElem(window)),
                    Err(ax_error) => Err(XcodeError::map_ax_error(ax_error)),
                }
            } else {
                Err(XcodeError::WindowHashUnknown)
            }
        }
        GetVia::Pid(pid) => {
            let application = AXUIElement::application(*pid);
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

#[cfg(test)]
mod tests {
    use accessibility::{AXUIElement, AXUIElementAttributes};

    use crate::ax_interaction::{internal::get_focused_uielement, GetVia, XcodeError};

    #[test]
    fn get_focused_uielement_via_current() {
        let system_wide_element = AXUIElement::system_wide();
        let focused_uielement = system_wide_element.focused_uielement();

        assert_eq!(
            get_focused_uielement(&GetVia::Current),
            focused_uielement.map_err(|err| XcodeError::map_ax_error(err))
        );
    }

    #[test]
    fn get_focused_uielement_via_pid() {
        let pid: i32 = std::process::id().try_into().unwrap();
        let application = AXUIElement::application(pid);
        let focused_uielement = application.focused_uielement();

        assert_eq!(
            get_focused_uielement(&GetVia::Pid(pid)),
            focused_uielement.map_err(|err| XcodeError::map_ax_error(err))
        );
    }
}
