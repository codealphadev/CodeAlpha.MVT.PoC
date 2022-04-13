use accessibility::{AXAttribute, AXUIElement, Error};
use cocoa::appkit::CGPoint;

use crate::{
    ax_interaction::{models::app::AppWindowMovedMessage, AXEventApp, AppObserverState},
    window_controls::AppWindow,
};

/// Notify Tauri that a window of our app has been moved
/// Method requires AXUIElement of type "AXWindow". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_moved(
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

    // Get updated window position
    let pos_ax_value = window_element.attribute(&AXAttribute::position())?;
    let origin = pos_ax_value.get_value::<CGPoint>()?;

    let msg = AppWindowMovedMessage {
        window: window,
        window_position: tauri::LogicalPosition {
            x: origin.x,
            y: origin.y,
        },
    };

    AXEventApp::AppWindowMoved(msg).publish_to_tauri(&app_state.app_handle);

    Ok(())
}
