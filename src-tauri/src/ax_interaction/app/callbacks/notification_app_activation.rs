use accessibility::{AXAttribute, AXUIElement, Error};
use enigo::Enigo;

use crate::{
    ax_interaction::{
        models::app::{AppActivatedMessage, AppDeactivatedMessage},
        AXEventApp, AppObserverState,
    },
    window_controls::{
        actions::{get_position, get_size},
        AppWindow,
    },
};

/// Notify Tauri that our app has been activated, which means focus has moved to our app from a different application.
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notifiy_app_activated(
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
pub fn notifiy_app_deactivated(
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
    let mut app_window: Option<AppWindow> = None;
    if let (Ok(position), Ok(size)) = (
        get_position(&app_handle, AppWindow::Widget),
        get_size(&app_handle, AppWindow::Widget),
    ) {
        let cursor_location: (i32, i32) = Enigo::mouse_location();

        if cursor_location.0 >= position.x as i32
            && cursor_location.0 <= position.x as i32 + size.width as i32
            && cursor_location.1 >= position.y as i32
            && cursor_location.1 <= position.y as i32 + size.height as i32
        {
            app_window = Some(AppWindow::Widget);
        }
    } else {
        app_window = None;
    }

    app_window
}
