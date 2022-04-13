use accessibility::{AXAttribute, AXUIElement, Error};

use crate::{
    ax_interaction::{models::app::AppWindowFocusedMessage, AXEventApp, AppObserverState},
    window_controls::AppWindow,
};

/// Notify Tauri that a window of our app has been focused
/// Method requires AXUIElement of type "AXWindow". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_focused(
    window_element: &AXUIElement,
    app_state: &AppObserverState,
) -> Result<(), Error> {
    let role = window_element.attribute(&AXAttribute::role())?;

    assert_eq!(role.to_string(), "AXWindow");

    let title = window_element.attribute(&AXAttribute::title())?;

    #[allow(unused_assignments)]
    let mut window = AppWindow::None;

    match title.to_string().as_str() {
        "CodeAlpha - Guide" => window = AppWindow::Content,
        "CodeAlpha - Settings" => window = AppWindow::Settings,
        "CodeAlpha - Analytics" => window = AppWindow::Analytics,
        "CodeAlpha - Widget" => window = AppWindow::Widget,
        _ => window = AppWindow::None,
    }

    let msg = AppWindowFocusedMessage { window: window };

    AXEventApp::AppWindowFocused(msg).publish_to_tauri(&app_state.app_handle);

    Ok(())
}
