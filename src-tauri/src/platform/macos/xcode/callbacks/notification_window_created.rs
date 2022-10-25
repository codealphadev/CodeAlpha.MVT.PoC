use crate::{
    platform::macos::{
        generate_axui_element_hash,
        internal::get_focused_uielement,
        models::editor::EditorWindowCreatedMessage,
        xcode::{callbacks::notify_uielement_focused, XCodeObserverState},
        AXEventXcode, GetVia,
    },
    utils::assert_or_error_trace,
};
use accessibility::{AXAttribute, AXUIElement, Error};
use accessibility_sys::kAXErrorInvalidUIElement;
use cocoa::appkit::CGPoint;
use core_foundation::base::{CFEqual, TCFType};
use core_graphics_types::geometry::CGSize;

/// Notify Tauri that an editor window has been created
/// Method requires AXUIElement of type "AXApplication". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_created(
    ui_element: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let mut app_element = ui_element.clone();
    if "AXWindow" == app_element.attribute(&AXAttribute::role())? {
        app_element = app_element.attribute(&AXAttribute::parent())?;
    } else if "AXApplication" != app_element.attribute(&AXAttribute::role())? {
        return Err(Error::Ax(kAXErrorInvalidUIElement));
    }

    let role = app_element.attribute(&AXAttribute::role())?;
    assert_or_error_trace(
        role.to_string() == "AXApplication",
        &format!(
            "notify_window_created() called with app_element of type {}; expected AXApplication",
            role.to_string()
        ),
    );

    let windows = app_element.attribute(&AXAttribute::children())?;

    for window in &windows {
        // Only add AXWindows
        if let Ok(role) = window.attribute(&AXAttribute::role()) {
            if role.to_string() != "AXWindow" {
                continue;
            }
        } else {
            continue;
        }

        // check if window is already contained in list of windows
        let window_exists = xcode_observer_state
            .window_list
            .iter()
            .any(|e| unsafe { CFEqual(window.as_CFTypeRef(), e.1.as_CFTypeRef()) != 0 });

        if !window_exists {
            let editor_name = app_element.attribute(&AXAttribute::title())?;
            let pid = app_element.pid()?;
            if let Ok(msg) = window_creation_msg(editor_name.to_string(), pid, &*window) {
                // Emit to rust listeners
                msg.publish_to_tauri(&xcode_observer_state.app_handle);

                // .... sorry, but ... extracting the window origin with a match statement.
                let window_origin = match msg {
                    AXEventXcode::EditorWindowCreated(msg) => tauri::LogicalPosition {
                        x: msg.window_position.x,
                        y: msg.window_position.y,
                    },
                    _ => tauri::LogicalPosition { x: 0.0, y: 0.0 },
                };

                // Add window to list of windows
                xcode_observer_state.window_list.push((
                    generate_axui_element_hash(&window),
                    window.clone(),
                    window_origin,
                    None,
                ));

                // Attempt to send an additional notification_uielement_focused
                if let Ok(element) = get_focused_uielement(&GetVia::Pid(pid)) {
                    let _ = notify_uielement_focused(&element, xcode_observer_state);
                }
            }
        }
    }

    Ok(())
}

fn window_creation_msg(
    editor_name: String,
    pid: i32,
    window_element: &AXUIElement,
) -> Result<AXEventXcode, Error> {
    let role = window_element.attribute(&AXAttribute::role())?;
    assert_or_error_trace(
        role.to_string() == "AXWindow",
        &format!(
            "window_creation_msg() called with window_element of type {}; expected AXWindow",
            role.to_string()
        ),
    );

    let size_ax_value = window_element.attribute(&AXAttribute::size())?;
    let pos_ax_value = window_element.attribute(&AXAttribute::position())?;

    let size = size_ax_value.get_value::<CGSize>()?;
    let origin = pos_ax_value.get_value::<CGPoint>()?;

    let window_created_msg = EditorWindowCreatedMessage {
        window_uid: generate_axui_element_hash(&window_element),
        window_position: tauri::LogicalPosition {
            x: origin.x,
            y: origin.y,
        },
        window_size: tauri::LogicalSize {
            width: size.width,
            height: size.height,
        },
        pid,
        editor_name,
    };

    Ok(AXEventXcode::EditorWindowCreated(window_created_msg))
}
