use accessibility::{AXAttribute, AXUIElement, Error};
use cocoa::appkit::CGPoint;
use core_foundation::base::{CFEqual, TCFType};
use core_graphics_types::geometry::CGSize;

use crate::ax_interaction::{
    models::editor::EditorWindowMovedMessage, AXEventXcode, XCodeObserverState,
};

/// Notify Tauri that an editor window has been moved
/// Method requires AXUIElement of type "AXWindow". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_moved(
    window_element: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let role = window_element.attribute(&AXAttribute::role())?;

    assert_eq!(role.to_string(), "AXWindow");

    // Find window_element in xcode_observer_state.window_list to get id
    let known_window = xcode_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = known_window {
        // Get updated window position and size
        let pos_ax_value = window_element.attribute(&AXAttribute::position())?;
        let size_ax_value = window_element.attribute(&AXAttribute::size())?;

        let origin = pos_ax_value.get_value::<CGPoint>()?;
        let size = size_ax_value.get_value::<CGSize>()?;

        // Publish to Tauri
        let msg = EditorWindowMovedMessage {
            id: window.0,
            window_position: tauri::LogicalPosition {
                x: origin.x,
                y: origin.y,
            },
            window_size: tauri::LogicalSize {
                width: size.width,
                height: size.height,
            },
        };

        AXEventXcode::EditorWindowMoved(msg)
            .publish_to_tauri(xcode_observer_state.app_handle.clone());
    }

    Ok(())
}
