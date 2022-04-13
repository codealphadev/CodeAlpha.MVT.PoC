use accessibility::{AXAttribute, AXUIElement, Error};

use crate::ax_interaction::{
    models::app::{AppActivatedMessage, AppDeactivatedMessage},
    AXEventApp,
};

/// Notify Tauri that our app has been activated, which means focus has moved to our app from a different application.
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notifiy_app_activated(
    app_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let role = app_element.attribute(&AXAttribute::role())?;
    assert_eq!(role.to_string(), "AXApplication");

    let name = app_element.attribute(&AXAttribute::title())?;
    let pid = app_element.pid()?;

    let activation_msg = AppActivatedMessage {
        app_name: name.to_string(),
        pid: pid.try_into().unwrap(),
    };

    let activation_event = AXEventApp::AppActivated(activation_msg);

    // Emit to rust listeners
    activation_event.publish_to_tauri(&app_handle);

    Ok(())
}

/// Notify Tauri that our app has been deactivated, which means focus has moved away from our app to a different application.
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notifiy_app_deactivated(
    app_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
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
    deactivation_event.publish_to_tauri(&app_handle);

    Ok(())
}
