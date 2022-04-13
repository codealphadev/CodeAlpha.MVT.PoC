use accessibility::{AXAttribute, AXUIElement, Error};

use crate::{
    ax_interaction::{models::app::AppWindowFocusedMessage, AXEventApp},
    window_controls::AppWindow,
};

/// Notify Tauri that a window of our app has been focused
/// Method requires AXUIElement of type "AXWindow". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_focused(
    window_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let role = window_element.attribute(&AXAttribute::role())?;

    assert_eq!(role.to_string(), "AXWindow");

    let title = window_element.attribute(&AXAttribute::title())?;
    let window: AppWindow = serde_json::from_str(&title.to_string()).unwrap();

    let msg = AppWindowFocusedMessage { window: window };

    AXEventApp::AppWindowFocused(msg).publish_to_tauri(app_handle);

    Ok(())
}
