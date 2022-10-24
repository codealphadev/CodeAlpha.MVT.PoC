use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use core_foundation::base::{CFEqual, TCFType};

use crate::platform::macos::{
    models::editor::EditorTextareaZoomedMessage, xcode::XCodeObserverState, AXEventXcode,
    EventViewport, GetVia,
};
use tracing::error;

pub fn notify_textarea_zoomed(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    if uielement.role()?.to_string() != "AXTextArea" {
        error!(
            "notify_textarea_zoomed() called with AXUIElement of type {}; expected AXTextArea",
            uielement.role()?.to_string()
        );
    }
    let window_element = uielement.window()?;

    // Find window_element in xcode_observer_state.window_list to get id
    let mut known_window = xcode_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = &mut known_window {
        AXEventXcode::EditorTextareaZoomed(EditorTextareaZoomedMessage {
            window_uid: window.0,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);

        // Publish an updated viewport properties message
        EventViewport::new_xcode_viewport_update(&GetVia::UIElem(window.1.clone()))
            .map_err(|_| accessibility::Error::NotFound)?
            .publish_to_tauri(&xcode_observer_state.app_handle)
    }

    Ok(())
}
