use crate::{
    platform::macos::{
        get_textarea_uielement, internal::get_uielement_frame,
        models::editor::EditorTextareaScrolledMessage, xcode::XCodeObserverState, AXEventXcode,
        EventViewport, GetVia,
    },
    window_controls::{
        models::editor_window::CodeOverlayDimensionsUpdateMessage, EventWindowControls,
    },
};
use accessibility::{AXUIElement, AXUIElementAttributes, Error};

use core_foundation::base::{CFEqual, TCFType};

pub fn notify_textarea_scrolled(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    assert_eq!(uielement.role()?, "AXScrollBar");

    let window_element = uielement.window()?;

    // Find window_element in xcode_observer_state.window_list to get id
    let mut known_window = xcode_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = &mut known_window {
        update_code_document_dimensions(&window_element, &xcode_observer_state.app_handle).ok();
        AXEventXcode::EditorTextareaScrolled(EditorTextareaScrolledMessage {
            window_uid: window.0,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);

        // Publish an updated viewport properties message
        EventViewport::new_xcode_viewport_update(&GetVia::UIElem(window.1.clone()))
            .map_err(|_| accessibility::Error::NotFound)?
            .publish_to_tauri(&xcode_observer_state.app_handle)
    }
    Ok(())
}

pub fn update_code_document_dimensions(
    window_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let code_document_uielement =
        get_textarea_uielement(&GetVia::Pid(window_element.pid()?)).map_err(|_| Error::NotFound)?;
    let code_document_frame =
        get_uielement_frame(&code_document_uielement).map_err(|_| Error::NotFound)?;
    // This event needs to arrive in the frontend before any annotation events
    // because the frontend relies on always having the correct document rect to handle global coordinates sent from core engine.
    EventWindowControls::CodeOverlayDimensionsUpdate(CodeOverlayDimensionsUpdateMessage {
        code_document_rect: code_document_frame,
        code_viewport_rect: None,
    })
    .publish_to_tauri(&app_handle);
    Ok(())
}
