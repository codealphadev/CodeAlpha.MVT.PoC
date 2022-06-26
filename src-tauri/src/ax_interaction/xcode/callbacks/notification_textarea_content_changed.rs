use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use core_foundation::{
    base::{CFEqual, TCFType},
    string::CFString,
};

use crate::ax_interaction::{
    models::editor::EditorTextareaContentChanged, AXEventXcode, XCodeObserverState,
};

/// It checks if the `AXUIElement`'s role is a `AXScrollBar` or a `AXTextArea` and if it is, it sends a
/// message to the Tauri app
///
/// Arguments:
///
/// * `uielement`: The element that has changed.
/// * `xcode_observer_state`: This is a mutable reference to the XCodeObserverState struct.
///
/// Returns:
///
/// A Result<(), Error>
pub fn notify_textarea_content_changed(
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
        let content = uielement.value()?;
        let content_str = content.downcast::<CFString>();

        if let Some(cf_str) = content_str {
            AXEventXcode::EditorTextareaContentChanged(EditorTextareaContentChanged {
                id: window.0,
                content: cf_str.to_string(),
            })
            .publish_to_tauri(&xcode_observer_state.app_handle);
        }
    }
    Ok(())
}