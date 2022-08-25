use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use core_foundation::base::{CFEqual, TCFType};

use crate::{
    app_handle,
    ax_interaction::{
        derive_xcode_textarea_dimensions, focused_uielement_of_app, get_textarea_frame,
        models::editor::EditorTextareaScrolledMessage, AXEventXcode, XCodeObserverState,
    },
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
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
        // TODO: Move to window controls? Or avoid extra event? Probably best to move.
        let textarea_uielement = focused_uielement_of_app(uielement.pid().unwrap())
            .ok()
            .unwrap(); // TODO: handle error and streamline
        let code_doc_rect = get_textarea_frame(&textarea_uielement).ok().unwrap();
        let view_doc_rect = derive_xcode_textarea_dimensions(&textarea_uielement).unwrap();

        EventWindowControls::CodeOverlayDimensionsUpdate(CodeOverlayDimensionsUpdateMessage {
            code_viewport_rect: LogicalFrame {
                origin: LogicalPosition::from_tauri_LogicalPosition(&view_doc_rect.0),
                size: LogicalSize::from_tauri_LogicalSize(&view_doc_rect.1),
            },
            code_document_rect: code_doc_rect,
        })
        .publish_to_tauri(&app_handle());
        AXEventXcode::EditorTextareaScrolled(EditorTextareaScrolledMessage {
            id: window.0,
            uielement_hash: window.3,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);
    }
    Ok(())
}
