use accessibility::{AXAttribute, AXUIElement, Error};
use tracing::error;

use crate::{
    platform::macos::{app::AppObserverState, models::app::AppWindowFocusedMessage, AXEventApp},
    window_controls::config::{default_properties, AppWindow},
};

/// Notify Tauri that a window of our app has been focused
/// Method requires AXUIElement of type "AXWindow". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_focused(
    window_element: &AXUIElement,
    app_state: &AppObserverState,
) -> Result<(), Error> {
    let role = window_element.attribute(&AXAttribute::role())?;

    if role.to_string() != "AXWindow" {
        error!(
            "notify_window_focused() called with window_element of type {}; expected AXWindow",
            role.to_string()
        );
    }

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
        let msg = AppWindowFocusedMessage { window: window };

        AXEventApp::AppWindowFocused(msg).publish_to_tauri(&app_state.app_handle);
    }

    Ok(())
}
