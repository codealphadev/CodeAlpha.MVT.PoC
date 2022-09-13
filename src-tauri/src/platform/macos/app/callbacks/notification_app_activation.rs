use accessibility::{AXAttribute, AXUIElement, Error};
use enigo::Enigo;
use tauri::Manager;

use crate::{
    platform::macos::{
        app::AppObserverState,
        models::app::{AppActivatedMessage, AppDeactivatedMessage},
        AXEventApp,
    },
    window_controls::config::AppWindow,
};

/// Notify Tauri that our app has been activated, which means focus has moved to our app from a different application.
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notify_app_activated(
    app_element: &AXUIElement,
    app_state: &AppObserverState,
) -> Result<(), Error> {
    let role = app_element.attribute(&AXAttribute::role())?;
    assert_eq!(role.to_string(), "AXApplication");

    let name = app_element.attribute(&AXAttribute::title())?;
    let pid = app_element.pid()?;

    let activation_msg = AppActivatedMessage {
        app_name: name.to_string(),
        pid: pid.try_into().unwrap(),
        focused_app_window: check_focus_on_widget_window(&app_state.app_handle),
    };

    let activation_event = AXEventApp::AppActivated(activation_msg);

    // Emit to rust listeners
    activation_event.publish_to_tauri(&app_state.app_handle);

    Ok(())
}

/// Notify Tauri that our app has been deactivated, which means focus has moved away from our app to a different application.
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notify_app_deactivated(
    app_element: &AXUIElement,
    app_state: &AppObserverState,
) -> Result<(), Error> {
    let role = app_element.attribute(&AXAttribute::role())?;
    assert_eq!(role.to_string(), "AXApplication");

    let name = app_element.attribute(&AXAttribute::title())?;
    let pid = app_element.pid()?;

    let deactivation_msg = AppDeactivatedMessage {
        app_name: name.to_string(),
        pid: pid.try_into().unwrap(),
    };

    let deactivation_event = AXEventApp::AppDeactivated(deactivation_msg);

    // Emit to rust listeners
    deactivation_event.publish_to_tauri(&app_state.app_handle);

    Ok(())
}

/// If the mouse cursor is inside the widget window, return Some(AppWindow::Widget), otherwise return
/// None
///
/// Arguments:
///
/// * `app_element`: The AXUIElement of the application.
/// * `app_state`: &AppObserverState
///
/// Returns:
///
/// An Option containing the focused AppWindow if mouse cursor is inside the widget window, otherwise None
fn check_focus_on_widget_window(app_handle: &tauri::AppHandle) -> Option<AppWindow> {
    let tauri_window = app_handle.get_window(&AppWindow::Widget.to_string())?;
    let monitor = tauri_window.current_monitor().ok()??;
    let scale_factor = monitor.scale_factor();
    let origin = tauri_window
        .outer_position()
        .ok()?
        .to_logical::<f64>(scale_factor);
    let size = tauri_window
        .outer_size()
        .ok()?
        .to_logical::<f64>(scale_factor);

    let cursor_location: (i32, i32) = Enigo::mouse_location();

    if cursor_location.0 >= origin.x as i32
        && cursor_location.0 <= origin.x as i32 + size.width as i32
        && cursor_location.1 >= origin.y as i32
        && cursor_location.1 <= origin.y as i32 + size.height as i32
    {
        Some(AppWindow::Widget)
    } else {
        None
    }
}
