use accessibility::{AXAttribute, AXUIElement, Error};
use cocoa::appkit::CGPoint;
use core_graphics_types::geometry::CGSize;

use crate::ax_events::{
    models::{XCodeFocusElement, XCodeFocusStatusChange},
    Event,
};
use crate::ax_interaction::utils::XCodeObserverState;

// This method is exectuted when the observer receives a notification of type "kAXFocusedUIElementChangedNotification"
// It means the use has clicked a different ui element WITHIN Xcode.
// This event is important, because we only want to show the widget when the user's curser is in the text area of the editor.
pub fn notification_focused_uielement(
    focused_element: &AXUIElement,
    xcode_observer_state: &XCodeObserverState,
) -> Result<(), Error> {
    let role = focused_element.attribute(&AXAttribute::role())?;

    if role.to_string().as_str() == "AXTextArea" {
        // Get the frame of the parent UI element --> TextAreas don't contain the actual coordinates of the window
        let parent_ui_element = focused_element.attribute(&AXAttribute::parent())?;

        let size_ax_val = parent_ui_element.attribute(&AXAttribute::size())?;
        let pos_ax_val = parent_ui_element.attribute(&AXAttribute::position())?;

        let size = size_ax_val.get_value::<CGSize>()?;
        let origin = pos_ax_val.get_value::<CGPoint>()?;

        let focus_change = XCodeFocusStatusChange {
            focus_element_change: XCodeFocusElement::Editor,
            is_in_focus: true,
            ui_element_x: origin.x,
            ui_element_y: origin.y,
            ui_element_w: size.width,
            ui_element_h: size.height,
        };

        let event = Event::XCodeFocusStatusChange(focus_change);
        event.publish_to_tauri(xcode_observer_state.app_handle.clone());
    } else {
        let focus_change = XCodeFocusStatusChange {
            focus_element_change: XCodeFocusElement::App,
            is_in_focus: true,
            ui_element_x: 0.0,
            ui_element_y: 0.0,
            ui_element_w: 0.0,
            ui_element_h: 0.0,
        };

        let event = Event::XCodeFocusStatusChange(focus_change);
        event.publish_to_tauri(xcode_observer_state.app_handle.clone());
    }

    Ok(())
}
