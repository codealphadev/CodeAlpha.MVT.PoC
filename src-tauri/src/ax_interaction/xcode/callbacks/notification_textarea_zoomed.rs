use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use core_foundation::base::{CFEqual, TCFType};

use crate::ax_interaction::{
    models::editor::EditorTextareaZoomedMessage, AXEventXcode, XCodeObserverState,
};

pub fn notify_textarea_zoomed(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    assert_eq!(uielement.role()?, "AXTextArea");

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
            id: window.0,
            uielement_hash: window.3,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);
    }

    Ok(())
}
