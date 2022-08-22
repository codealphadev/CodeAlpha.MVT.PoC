use accessibility::{AXAttribute, AXUIElement, Error};

use crate::ax_interaction::{
    models::editor::{EditorAppActivatedMessage, EditorAppDeactivatedMessage},
    AXEventXcode, XCodeObserverState,
};

/// Notify Tauri that XCode has been activated, which means focus has moved to XCode from a different application.
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notify_app_activated(
    app_element: &AXUIElement,
    xcode_observer_state: &XCodeObserverState,
) -> Result<(), Error> {
    let role = app_element.attribute(&AXAttribute::role())?;
    assert_eq!(role.to_string(), "AXApplication");

    let name = app_element.attribute(&AXAttribute::title())?;
    let pid = app_element.pid()?;

    let activation_msg = EditorAppActivatedMessage {
        editor_name: name.to_string(),
        pid: pid.try_into().unwrap(),
    };

    let activation_event = AXEventXcode::EditorAppActivated(activation_msg);

    // Emit to rust listeners
    activation_event.publish_to_tauri(&xcode_observer_state.app_handle);

    Ok(())
}

/// Notify Tauri that XCode has been deactivated, which means focus has moved away from XCode to a different application.
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notify_app_deactivated(
    app_element: &AXUIElement,
    xcode_observer_state: &XCodeObserverState,
) -> Result<(), Error> {
    let role = app_element.attribute(&AXAttribute::role())?;
    assert_eq!(role.to_string(), "AXApplication");

    let name = app_element.attribute(&AXAttribute::title())?;
    let pid = app_element.pid()?;

    let deactivation_msg = EditorAppDeactivatedMessage {
        editor_name: name.to_string(),
        pid: pid.try_into().unwrap(),
    };

    let deactivation_event = AXEventXcode::EditorAppDeactivated(deactivation_msg);

    // Emit to rust listeners
    deactivation_event.publish_to_tauri(&xcode_observer_state.app_handle);

    Ok(())
}
