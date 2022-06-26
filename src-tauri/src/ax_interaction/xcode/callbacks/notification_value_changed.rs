use accessibility::{AXUIElement, AXUIElementAttributes, Error};

use crate::ax_interaction::XCodeObserverState;

use super::{
    notifiy_textarea_scrolled, notifiy_textarea_zoomed, notify_textarea_content_changed,
    notify_window_resized,
};

pub fn notify_value_changed(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let uielement_role = uielement.role()?;

    match uielement_role.to_string().as_str() {
        "AXScrollBar" => {
            notify_window_resized(&uielement, &mut (*xcode_observer_state))?;
            notifiy_textarea_scrolled(&uielement, &mut (*xcode_observer_state))?;

            Ok(())
        }
        "AXTextArea" => {
            // Detect edge case: we also land here when the user zooms the textarea in/out. The ui element
            // sending a value changed event is seemingly of type TextAre, but does not contain the editor's
            // textarea content.
            //
            // Interestingly, the marker to identify this case is the presence of a children with role "AXTextArea"
            // of the UI element's parent. This does not really make sense, because we end up in this match branch,
            // the parent should ALWAYS contain an AXTextArea child. :shrug:

            let parent = uielement.parent()?;
            let children = parent.children()?;

            let mut found_textarea_is_editor_textarea = false;
            for child in &children {
                let child_role = child.role()?;
                if child_role.to_string().as_str() == "AXTextArea" {
                    found_textarea_is_editor_textarea = true;
                    break;
                }
            }

            if found_textarea_is_editor_textarea {
                notify_textarea_content_changed(&uielement, &mut (*xcode_observer_state))
            } else {
                notifiy_textarea_zoomed(uielement, &mut (*xcode_observer_state))
            }
        }
        _ => Ok(()),
    }
}
