use crate::{
    platform::macos::{app::AppObserverState, models::app::AppWindowMovedMessage, AXEventApp},
    utils::assert_or_error_trace,
    window_controls::config::{default_properties, AppWindow},
};
use accessibility::{AXAttribute, AXUIElement, Error};
use cocoa::appkit::CGPoint;

/// Notify Tauri that a window of our app has been moved
/// Method requires AXUIElement of type "AXWindow". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_moved(
    window_element: &AXUIElement,
    app_state: &AppObserverState,
) -> Result<(), Error> {
    let role = window_element.attribute(&AXAttribute::role())?;

    assert_or_error_trace(
        role.to_string() == "AXWindow",
        &format!(
            "notify_window_moved() called with window_element of type {}; expected AXWindow",
            role.to_string()
        ),
    );

    let title = window_element.attribute(&AXAttribute::title())?;

    use strum::IntoEnumIterator;

    let mut window: Option<AppWindow> = None;
    for app_window in AppWindow::iter() {
        if title.to_string() == default_properties::title(&app_window) {
            window = Some(app_window);
            break;
        }
    }

    // Get updated window position
    let pos_ax_value = window_element.attribute(&AXAttribute::position())?;
    let origin = pos_ax_value.get_value::<CGPoint>()?;

    if let Some(window) = window {
        let msg = AppWindowMovedMessage {
            window: window,
            window_position: tauri::LogicalPosition {
                x: origin.x,
                y: origin.y,
            },
        };

        AXEventApp::AppWindowMoved(msg).publish_to_tauri(&app_state.app_handle);
    }

    Ok(())
}
