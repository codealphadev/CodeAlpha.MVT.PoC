use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use core_foundation::base::{CFEqual, TCFType};

use crate::{
    ax_interaction::{
        get_textarea_frame, get_textarea_uielement, models::editor::EditorTextareaScrolledMessage,
        AXEventXcode, XCodeObserverState,
    },
    window_controls::{
        models::editor_window::CodeOverlayDimensionsUpdateMessage, EventWindowControls,
    },
};

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
            id: window.0,
            uielement_hash: window.3,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);
    }
    Ok(())
}

pub fn update_code_document_dimensions(
    window_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let textarea_uielement_opt = get_textarea_uielement(window_element.pid()?);
    if let Some(textarea_uielement) = textarea_uielement_opt {
        let code_doc_rect = get_textarea_frame(&textarea_uielement)?;
        // This event needs to arrive in the frontend before any annotation events
        // because the frontend relies on always having the correct document rect to handle global coordinates sent from core engine.
        EventWindowControls::CodeOverlayDimensionsUpdate(CodeOverlayDimensionsUpdateMessage {
            code_viewport_rect: None,
            code_document_rect: code_doc_rect,
        })
        .publish_to_tauri(&app_handle);
    }
    Ok(())
}
