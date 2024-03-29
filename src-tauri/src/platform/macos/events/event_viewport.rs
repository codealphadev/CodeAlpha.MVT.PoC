use std::fmt;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::{
    platform::macos::{
        get_code_document_frame_properties, get_textarea_uielement, get_viewport_properties,
        internal::get_uielement_frame, CodeDocumentFrameProperties, GetVia, ViewportProperties,
        XcodeError, LAST_KNOWN_VIEWPORT_FRAME,
    },
    utils::messaging::ChannelList,
    window_controls::config::AppWindow,
};

use super::models::viewport::ViewportPropertiesUpdateMessage;

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[serde(tag = "event", content = "payload")]
#[ts(export, export_to = "bindings/macOS_specific/")]
pub enum EventViewport {
    XcodeViewportUpdate(ViewportPropertiesUpdateMessage),
}

impl fmt::Display for EventViewport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventViewport::XcodeViewportUpdate(_) => write!(f, "ViewportPropertiesUpdateMessage"),
        }
    }
}

impl EventViewport {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventViewport.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to CodeOverlay window
        _ = app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to Main window
        _ = app_handle.emit_to(
            &AppWindow::Main.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        // Emit to Widget window
        _ = app_handle.emit_to(
            &AppWindow::Widget.to_string(),
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }

    pub fn new_xcode_viewport_update_minimal(get_via: &GetVia) -> Result<Self, XcodeError> {
        let textarea_uielement = get_textarea_uielement(get_via)?;
        let code_doc_frame = get_uielement_frame(&textarea_uielement)?;

        let viewport = LAST_KNOWN_VIEWPORT_FRAME
            .lock()
            .ok_or(XcodeError::GenericError(anyhow!(
                "Viewport frame not known."
            )))?;

        Ok(Self::XcodeViewportUpdate(ViewportPropertiesUpdateMessage {
            viewport_properties: ViewportProperties {
                dimensions: viewport.1,
                annotation_section: None,
                code_section: None,
                window_uid: viewport.0,
            },
            code_document_frame_properties: CodeDocumentFrameProperties {
                dimensions: code_doc_frame,
                text_offset: None,
            },
        }))
    }

    pub fn new_xcode_viewport_update(get_via: &GetVia) -> Result<Self, XcodeError> {
        Ok(Self::XcodeViewportUpdate(ViewportPropertiesUpdateMessage {
            viewport_properties: get_viewport_properties(&get_via)?,
            code_document_frame_properties: get_code_document_frame_properties(&get_via)?,
        }))
    }
}
