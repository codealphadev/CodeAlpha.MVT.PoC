use accessibility::{AXAttribute, AXUIElement, Error};

use crate::{
    ax_interaction::{
        models::app::{AppUIElementFocusedMessage, FocusedAppUIElement},
        AXEventApp, AppObserverState,
    },
    window_controls::config::AppWindow,
};

/// Notify Tauri that a new uielement in a window of our app has been focused
pub fn notify_uielement_focused(
    uielement_element: &AXUIElement,
    app_state: &AppObserverState,
) -> Result<(), Error> {
    let window_element = uielement_element.attribute(&AXAttribute::window())?;

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

    let uielement_focused_msg = AppUIElementFocusedMessage {
        focused_ui_element: FocusedAppUIElement::Other,
        window: window,
    };

    AXEventApp::AppUIElementFocused(uielement_focused_msg).publish_to_tauri(&app_state.app_handle);

    Ok(())
}
