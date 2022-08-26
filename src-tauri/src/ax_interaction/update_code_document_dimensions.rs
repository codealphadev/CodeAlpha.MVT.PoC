use accessibility::{AXUIElement, Error};

use crate::window_controls::{
    models::editor_window::CodeOverlayDimensionsUpdateMessage, EventWindowControls,
};

use super::{get_textarea_frame, get_textarea_uielement};

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
