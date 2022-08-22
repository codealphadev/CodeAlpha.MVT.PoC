use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use core_foundation::base::{CFEqual, CFRange, TCFType};

use crate::ax_interaction::{
    models::editor::EditorTextareaSelectedTextChangedMessage, AXEventXcode, XCodeObserverState,
};

pub fn notify_textarea_selected_text_changed(
    uielement: &AXUIElement,
    uielement_textarea: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    assert_eq!(uielement.role()?, "AXStaticText");

    let window_element = uielement.window()?;

    // Find window_element in xcode_observer_state.window_list to get id
    let mut known_window = xcode_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = &mut known_window {
        let text_range = uielement_textarea.selected_text_range()?;
        let text_range_str = text_range.get_value::<CFRange>()?;

        AXEventXcode::EditorTextareaSelectedTextChanged(EditorTextareaSelectedTextChangedMessage {
            id: window.0,
            index: text_range_str.location as usize,
            length: text_range_str.length as usize,
            selected_text: (uielement_textarea.selected_text()?).to_string(),
            ui_elem_hash: window.3,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);
    }

    Ok(())
}
