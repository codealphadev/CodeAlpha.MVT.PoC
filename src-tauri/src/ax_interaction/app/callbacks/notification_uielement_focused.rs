use accessibility::{AXAttribute, AXUIElement, Error};

use crate::{
    ax_interaction::{
        models::app::{AppUIElementFocusedMessage, FocusedAppUIElement},
        AXEventApp,
    },
    window_controls::AppWindow,
};

/// Notify Tauri that a new uielement in a window of our app has been focused
pub fn notify_uielement_focused(
    uielement_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let window_element = uielement_element.attribute(&AXAttribute::window())?;

    let title = window_element.attribute(&AXAttribute::title())?;
    let window: AppWindow = serde_json::from_str(&title.to_string()).unwrap();

    let uielement_focused_msg = AppUIElementFocusedMessage {
        focused_ui_element: FocusedAppUIElement::Other,
        window: window,
    };

    AXEventApp::AppUIElementFocused(uielement_focused_msg).publish_to_tauri(app_handle);

    Ok(())
}
