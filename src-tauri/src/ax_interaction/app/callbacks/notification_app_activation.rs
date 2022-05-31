use accessibility::{AXAttribute, AXUIElement, AXUIElementAttributes, Error};

use crate::{
    ax_interaction::{
        models::app::{AppActivatedMessage, AppDeactivatedMessage},
        AXEventApp, AppObserverState,
    },
    window_controls::AppWindow,
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

    // attempt to get the currently focused window
    let mut app_window: Option<AppWindow> = None;
    if let Ok(focused_element) = app_element.focused_uielement() {
        if let Ok(window_element) = focused_element.window() {
            if let Ok(title) = window_element.title() {
                #[allow(unused_assignments)]
                match title.to_string().as_str() {
                    "CodeAlpha - Guide" => app_window = Some(AppWindow::Content),
                    "CodeAlpha - Settings" => app_window = Some(AppWindow::Settings),
                    "CodeAlpha - Analytics" => app_window = Some(AppWindow::Analytics),
                    "CodeAlpha - Widget" => app_window = Some(AppWindow::Widget),
                    _ => app_window = Some(AppWindow::None),
                }
            } else {
                app_window = Some(AppWindow::None);
            }
        }
    }

    let activation_msg = AppActivatedMessage {
        app_name: name.to_string(),
        pid: pid.try_into().unwrap(),
        focused_app_window: app_window,
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
