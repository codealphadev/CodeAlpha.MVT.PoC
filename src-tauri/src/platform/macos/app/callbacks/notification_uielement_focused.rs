use accessibility::{AXAttribute, AXUIElement, Error};

use crate::{
    platform::macos::{
        app::AppObserverState,
        models::app::{AppUIElementFocusedMessage, FocusedAppUIElement},
        AXEventApp,
    },
    window_controls::config::{default_properties, AppWindow},
};

/// Notify Tauri that a new uielement in a window of our app has been focused
pub fn notify_uielement_focused(
    uielement_element: &AXUIElement,
    app_state: &AppObserverState,
) -> Result<(), Error> {
    let window_element = uielement_element.attribute(&AXAttribute::window())?;

    let title = window_element.attribute(&AXAttribute::title())?;

    use strum::IntoEnumIterator;

    let mut window: Option<AppWindow> = None;
    for app_window in AppWindow::iter() {
        if title.to_string() == default_properties::title(&app_window) {
            window = Some(app_window);
            break;
        }
    }

    if let Some(window) = window {
        let uielement_focused_msg = AppUIElementFocusedMessage {
            focused_ui_element: FocusedAppUIElement::Other,
            window: window,
        };

        AXEventApp::AppUIElementFocused(uielement_focused_msg)
            .publish_to_tauri(&app_state.app_handle);
    }

    Ok(())
}
