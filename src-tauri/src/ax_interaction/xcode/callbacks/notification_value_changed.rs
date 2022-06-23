use accessibility::{AXUIElement, AXUIElementAttributes, Error};

use crate::ax_interaction::XCodeObserverState;

use super::{notifiy_textarea_scrolled, notify_textarea_content_changed, notify_window_resized};

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
        "AXTextArea" => notify_textarea_content_changed(&uielement, &mut (*xcode_observer_state)),
        _ => Ok(()),
    }
}
