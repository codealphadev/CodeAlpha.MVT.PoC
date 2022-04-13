use accessibility::{AXAttribute, AXUIElement, Error};
use cocoa::appkit::CGPoint;

use crate::{
    ax_interaction::{models::app::AppWindowMovedMessage, AXEventApp},
    window_controls::AppWindow,
};

/// Notify Tauri that a window of our app has been moved
/// Method requires AXUIElement of type "AXWindow". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_moved(
    window_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let role = window_element.attribute(&AXAttribute::role())?;

    assert_eq!(role.to_string(), "AXWindow");

    let title = window_element.attribute(&AXAttribute::title())?;
    let window: AppWindow = serde_json::from_str(&title.to_string()).unwrap();

    // Get updated window position and size
    let pos_ax_value = window_element.attribute(&AXAttribute::position())?;
    let origin = pos_ax_value.get_value::<CGPoint>()?;

    let msg = AppWindowMovedMessage {
        window: window,
        window_position: tauri::LogicalPosition {
            x: origin.x,
            y: origin.y,
        },
    };

    AXEventApp::AppWindowMoved(msg).publish_to_tauri(&app_handle);

    Ok(())
}
