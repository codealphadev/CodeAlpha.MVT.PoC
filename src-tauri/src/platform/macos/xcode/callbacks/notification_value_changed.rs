use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use core_foundation::string::CFString;

use crate::{
    app_handle,
    platform::macos::{
        get_minimal_viewport_properties, get_textarea_uielement,
        models::editor::{EditorUIElementFocusedMessage, FocusedUIElement},
        xcode::{callbacks::notify_textarea_selected_text_changed, XCodeObserverState},
        AXEventXcode, GetVia,
    },
};

use super::{
    notify_textarea_content_changed, notify_textarea_scrolled, notify_textarea_zoomed,
    notify_window_resized,
};

pub fn notify_value_changed(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    let uielement_role = uielement.role()?;
    match uielement_role.to_string().as_str() {
        "AXScrollBar" => {
            // Because the textarea can also be resized without resizing the editor window, we need to detect this case
            // using a different notification - this one.
            notify_window_resized(&uielement, &mut (*xcode_observer_state))?;

            notify_textarea_scrolled(&uielement, &mut (*xcode_observer_state))?;
            Ok(())
        }
        "AXStaticText" => {
            let uielement_textarea = get_textarea_uielement(&GetVia::Pid(uielement.pid()?));

            if let Ok(uielement_textarea) = uielement_textarea {
                notify_textarea_selected_text_changed(
                    &uielement,
                    &uielement_textarea,
                    &mut (*xcode_observer_state),
                )?;
            }

            check_event_received_due_to_xcode_dev_panel_closed(&uielement);

            Ok(())
        }
        "AXTextArea" => {
            let found_textarea_is_editor_textarea =
                uielement.description()?.to_string() == "Source Editor";

            if found_textarea_is_editor_textarea {
                notify_textarea_content_changed(&uielement, &mut (*xcode_observer_state))
            } else {
                notify_textarea_zoomed(uielement, &mut (*xcode_observer_state))
            }
        }
        _ => Ok(()),
    }
}

fn check_event_received_due_to_xcode_dev_panel_closed(uielement: &AXUIElement) {
    // Case: Observed issue COD-282; Removing Split-Panel in the editor does not reset CodeOverlay window
    // Closing a split-dev-panel does not emit a notification that the UIElementFocus has changed. Instead, we
    // observed `XCode notification: "AXValueChanged", ui element role: "AXStaticText", value: "No Selection")`
    // This literally tells us in "text" what Xcode should normally tell us by emitting a "UIElementFocusChanged" notification.
    if let Ok(uielement_role) = uielement.role() {
        if uielement_role.to_string() == "AXStaticText" {
            if let Ok(uielement_value) = uielement.value() {
                if let Some(uielement_value_str) = uielement_value.downcast::<CFString>() {
                    if uielement_value_str.to_string() == "No Selection" {
                        let (viewport_props, code_doc_props) =
                            if let Ok((viewport_props, code_doc_props)) =
                                get_minimal_viewport_properties(&GetVia::Current)
                            {
                                (Some(viewport_props), Some(code_doc_props))
                            } else {
                                (None, None)
                            };

                        AXEventXcode::EditorUIElementFocused(EditorUIElementFocusedMessage {
                            window_uid: None,
                            pid: None,
                            focused_ui_element: FocusedUIElement::Other,
                            textarea_position: None,
                            textarea_size: None,
                            viewport: viewport_props,
                            code_document: code_doc_props,
                        })
                        .publish_to_tauri(&app_handle());
                    }
                }
            }
        }
    }
}
