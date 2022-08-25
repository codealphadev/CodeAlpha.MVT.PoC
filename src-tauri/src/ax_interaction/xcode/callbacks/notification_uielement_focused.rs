use accessibility::{AXAttribute, AXUIElement, Error};
use core_foundation::base::{CFEqual, TCFType};

use crate::ax_interaction::{
    models::editor::{EditorUIElementFocusedMessage, FocusedUIElement},
    xcode::XCodeObserverState,
    AXEventXcode,
};

/// Notify Tauri that an new uielement in an editor window has been focused
/// If the newly focused uielement is a textarea, the optional position and size of the
/// textarea will be included in the message
pub fn notify_uielement_focused(
    uielement_element: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let window_element = if let Ok(window) = uielement_element.attribute(&AXAttribute::window()) {
        window
    } else {
        AXEventXcode::EditorUIElementFocused(EditorUIElementFocusedMessage {
            window_id: None,
            focused_ui_element: FocusedUIElement::Other,
            textarea_position: None,
            textarea_size: None,
            ui_elem_hash: None,
            pid: None,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);
        return Ok(());
    };

    // Find window_element in xcode_observer_state.window_list to get id
    let known_window = xcode_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = known_window {
        let mut uielement_focused_msg = EditorUIElementFocusedMessage {
            window_id: Some(window.0),
            focused_ui_element: FocusedUIElement::Other,
            textarea_position: None,
            textarea_size: None,
            ui_elem_hash: Some(window.3),
            pid: Some(window.1.pid()?),
        };

        let role = uielement_element.attribute(&AXAttribute::role())?;

        // Some ui elements get focused programatically that don't mean anything to the user.
        // We skip those
        if role.to_string() == "AXSplitGroup" {
            return Ok(());
        }

        if role.to_string() == "AXTextArea" {
            let (position, size) = derive_xcode_textarea_dimensions(uielement_element)?;

            uielement_focused_msg.focused_ui_element = FocusedUIElement::Textarea;
            uielement_focused_msg.textarea_position = Some(position);
            uielement_focused_msg.textarea_size = Some(size);

            // update the window's textarea size
            let new_tuple = (window.0, window.1.clone(), Some(size), window.3);

            // Remove item window_list
            xcode_observer_state
                .window_list
                .retain(|vec_elem| vec_elem.0 != new_tuple.0);

            xcode_observer_state.window_list.push(new_tuple);
        }

        AXEventXcode::EditorUIElementFocused(uielement_focused_msg)
            .publish_to_tauri(&xcode_observer_state.app_handle);
    }

    Ok(())
}
