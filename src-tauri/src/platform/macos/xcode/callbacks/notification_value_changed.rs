use accessibility::{AXUIElement, AXUIElementAttributes, Error};

use crate::platform::macos::{
    get_textarea_uielement,
    xcode::{callbacks::notify_textarea_selected_text_changed, XCodeObserverState},
    GetVia,
};

use super::{
    notify_textarea_content_changed, notify_textarea_scrolled, notify_textarea_zoomed,
    notify_window_resized,
};

pub fn notify_value_changed(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let uielement_role = uielement.role()?;
    match uielement_role.to_string().as_str() {
        "AXScrollBar" => {
            // Because the textarea can also be resized without resizing the editor window, we need to detect this case
            // using a different notification - this one.
            notify_window_resized(&uielement, &mut (*xcode_observer_state))?;

            notify_textarea_scrolled(&uielement, &mut (*xcode_observer_state))?;
            Ok(())
        }
        "AXStaticText" => {
            let uielement_textarea = get_textarea_uielement(&GetVia::Pid(uielement.pid()?));

            if let Ok(uielement_textarea) = uielement_textarea {
                notify_textarea_selected_text_changed(
                    &uielement,
                    &uielement_textarea,
                    &mut (*xcode_observer_state),
                )?;
            }

            Ok(())
        }
        "AXTextArea" => {
            let found_textarea_is_editor_textarea =
                uielement.description()?.to_string() == "Source Editor";

            if found_textarea_is_editor_textarea {
                notify_textarea_content_changed(&uielement, &mut (*xcode_observer_state))
            } else {
                notify_textarea_zoomed(uielement, &mut (*xcode_observer_state))
            }
        }
        _ => Ok(()),
    }
}
