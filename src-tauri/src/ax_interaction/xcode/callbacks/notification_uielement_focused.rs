use accessibility::{AXAttribute, AXUIElement, Error};
use core_foundation::base::{CFEqual, TCFType};

use crate::ax_interaction::{
    models::editor::{EditorUIElementFocusedMessage, FocusedUIElement},
    AXEventXcode, XCodeObserverState,
};

use super::notification_window_resized::derive_textarea_dimensions;

/// Notify Tauri that an new uielement in an editor window has been focused
/// If the newly focused uielement is a textarea, the optional position and size of the
/// textarea will be included in the message
pub fn notify_uielement_focused(
    uielement_element: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let window_element = uielement_element.attribute(&AXAttribute::window())?;

    // Find window_element in xcode_observer_state.window_list to get id
    let known_window = xcode_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = known_window {
        let mut uielement_focused_msg = EditorUIElementFocusedMessage {
            window_id: window.0,
            focused_ui_element: FocusedUIElement::Other,
            textarea_position: None,
            textarea_size: None,
        };

        let role = uielement_element.attribute(&AXAttribute::role())?;
        if role.to_string() == "AXTextArea" {
            let (position, size) = derive_textarea_dimensions(uielement_element)?;

            uielement_focused_msg.focused_ui_element = FocusedUIElement::Textarea;
            uielement_focused_msg.textarea_position = Some(position);
            uielement_focused_msg.textarea_size = Some(size);
        }

        AXEventXcode::EditorUIElementFocused(uielement_focused_msg)
            .publish_to_tauri(&xcode_observer_state.app_handle);
    }

    Ok(())
}
