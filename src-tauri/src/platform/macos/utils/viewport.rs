use accessibility::{AXAttribute, AXUIElement};
use anyhow::anyhow;
use core_foundation::array::CFArray;
use core_graphics::geometry::CGRect;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::WindowUid,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
};

use super::{
    ax_helpers::ax_attribute, get_focused_window, get_textarea_uielement,
    internal::get_uielement_frame, GetVia, XcodeError,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/macOS_specific/xcode/")]
pub struct ViewportProperties {
    pub window_uid: WindowUid,
    pub dimensions: LogicalFrame,
    pub annotation_section: Option<LogicalFrame>,
    pub code_section: Option<LogicalFrame>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/macOS_specific/xcode/")]
pub struct CodeDocumentFrameProperties {
    pub dimensions: LogicalFrame,
    pub text_offset: Option<f64>,
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref LAST_KNOWN_VIEWPORT_FRAME: parking_lot::Mutex<Option<(WindowUid, LogicalFrame)>> =
        parking_lot::Mutex::new(None);
}

pub fn get_code_document_frame_properties(
    get_via: &GetVia,
) -> Result<CodeDocumentFrameProperties, XcodeError> {
    let code_document_uielement = get_textarea_uielement(get_via)?;

    Ok(CodeDocumentFrameProperties {
        dimensions: get_uielement_frame(&code_document_uielement)?,
        text_offset: Some(get_text_offset_px(&get_via)?),
    })
}

pub fn get_viewport_properties(get_via: &GetVia) -> Result<ViewportProperties, XcodeError> {
    let annotation_section = get_annotation_section_frame(&get_via)?;
    let code_section = get_code_section_frame(&get_via)?;

    let diff_origin_x = code_section.origin.x - annotation_section.origin.x;

    let viewport_frame = LogicalFrame {
        origin: annotation_section.origin,
        size: LogicalSize {
            width: diff_origin_x + code_section.size.width,
            height: annotation_section.size.height,
        },
    };

    let window_uid = get_focused_window()?;

    LAST_KNOWN_VIEWPORT_FRAME
        .lock()
        .replace((window_uid.clone(), viewport_frame.clone()));

    Ok(ViewportProperties {
        dimensions: viewport_frame,
        annotation_section: Some(annotation_section),
        code_section: Some(code_section),
        window_uid,
    })
}

pub fn get_viewport_frame(get_via: &GetVia) -> Result<LogicalFrame, XcodeError> {
    let viewport_properties = get_viewport_properties(get_via)?;
    Ok(viewport_properties.dimensions)
}

pub fn get_annotation_section_frame(get_via: &GetVia) -> Result<LogicalFrame, XcodeError> {
    let scrollarea_uielement = get_scrollarea_uielement(get_via)?;

    let text_offset = get_text_offset_px(get_via)?;
    let annotation_origin_x_offset = get_annotation_origin_x_offset_px(&scrollarea_uielement)?;

    let scrollarea = get_uielement_frame(&scrollarea_uielement)?;

    Ok(LogicalFrame {
        origin: LogicalPosition {
            x: scrollarea.origin.x + annotation_origin_x_offset,
            y: scrollarea.origin.y,
        },
        size: LogicalSize {
            width: text_offset - annotation_origin_x_offset,
            height: scrollarea.size.height,
        },
    })
}

pub fn get_code_section_frame(get_via: &GetVia) -> Result<LogicalFrame, XcodeError> {
    let scrollarea_uielement = get_scrollarea_uielement(get_via)?;

    let scrollarea_frame = get_uielement_frame(&scrollarea_uielement)?;

    // Get all children
    let children_elements = ax_attribute(&scrollarea_uielement, AXAttribute::children())?;

    let minimap_frame = get_viewport_minimap_frame(&children_elements).ok();
    let gutter_frame = get_viewport_gutter_frame(&children_elements)?;
    let gutter_change_frame = get_viewport_gutter_change_frame(&children_elements)?;

    let optional_folding_ribbon_frame = get_viewport_folding_ribbon_frame(&children_elements).ok();

    Ok(LogicalFrame {
        origin: LogicalPosition {
            x: scrollarea_frame.origin.x
                + gutter_frame.size.width
                + gutter_change_frame.size.width
                + optional_folding_ribbon_frame
                    .map_or(0.0, |folding_ribbon_frame| folding_ribbon_frame.size.width),
            y: scrollarea_frame.origin.y,
        },
        size: LogicalSize {
            width: scrollarea_frame.size.width
                - gutter_frame.size.width
                - gutter_change_frame.size.width
                - optional_folding_ribbon_frame
                    .map_or(0.0, |folding_ribbon_frame| folding_ribbon_frame.size.width)
                - minimap_frame.map_or(0.0, |minimap_frame| minimap_frame.size.width),
            height: scrollarea_frame.size.height,
        },
    })
}

fn get_text_offset_px(get_via: &GetVia) -> Result<f64, XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;

    let code_document_frame = get_uielement_frame(&textarea_uielement)?;

    let visible_character_range =
        ax_attribute(&textarea_uielement, AXAttribute::visible_character_range())?;

    if let Ok(character_range_bounds_axval) = textarea_uielement
        .parameterized_attribute(&AXAttribute::bounds_for_range(), &visible_character_range)
    {
        if let Ok(character_range_bounds) = character_range_bounds_axval.get_value::<CGRect>() {
            let text_offset_px = character_range_bounds.origin.x - code_document_frame.origin.x;
            if text_offset_px < 0.0 {
                return Err(XcodeError::ImplausibleDimensions);
            } else {
                return Ok(text_offset_px);
            }
        }
    }

    Err(XcodeError::GettingTextContentFailed)
}

fn get_annotation_origin_x_offset_px(viewport_uielement: &AXUIElement) -> Result<f64, XcodeError> {
    let viewport_children = ax_attribute(&viewport_uielement, AXAttribute::children())?;

    let gutter_frame = get_viewport_gutter_frame(&viewport_children)?;
    let change_gutter_frame = get_viewport_gutter_change_frame(&viewport_children)?;
    let optional_folding_ribbon_frame = get_viewport_folding_ribbon_frame(&viewport_children).ok();

    // We make the textarea a little bit bigger so our annotations have more space to draw on
    let correction_width_factor = 0.105;

    Ok(gutter_frame.size.width
        + change_gutter_frame.size.width
        + optional_folding_ribbon_frame.map_or(
            -(correction_width_factor * gutter_frame.size.width),
            |folding_ribbon_frame| folding_ribbon_frame.size.width,
        ))
}

fn get_scrollarea_uielement(get_via: &GetVia) -> Result<AXUIElement, XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;

    ax_attribute(&textarea_uielement, AXAttribute::parent())
}

fn get_viewport_minimap_frame(
    viewport_children: &CFArray<AXUIElement>,
) -> Result<LogicalFrame, XcodeError> {
    // Get all children

    for child in viewport_children {
        if let Ok(identifier) = child.attribute(&AXAttribute::identifier()) {
            if identifier.to_string().as_str() == "Source Editor Minimap" {
                return get_uielement_frame(&child);
            }
        }
    }

    Err(XcodeError::UIElementNotFound)
}

fn get_viewport_gutter_frame(
    viewport_children: &CFArray<AXUIElement>,
) -> Result<LogicalFrame, XcodeError> {
    // Get all children

    for child in viewport_children {
        if let Ok(identifier) = child.attribute(&AXAttribute::identifier()) {
            if identifier.to_string().as_str() == "Source Editor Gutter" {
                return get_uielement_frame(&child);
            }
        }
    }

    Err(XcodeError::UIElementNotFound)
}

fn get_viewport_gutter_change_frame(
    viewport_children: &CFArray<AXUIElement>,
) -> Result<LogicalFrame, XcodeError> {
    // Get all children

    for child in viewport_children {
        if let Ok(identifier) = child.attribute(&AXAttribute::identifier()) {
            if identifier.to_string().as_str() == "Source Editor Change Gutter" {
                return get_uielement_frame(&child);
            }
        }
    }

    Err(XcodeError::UIElementNotFound)
}

fn get_viewport_folding_ribbon_frame(
    viewport_children: &CFArray<AXUIElement>,
) -> Result<LogicalFrame, XcodeError> {
    // Get all children

    for child in viewport_children {
        if let Ok(identifier) = child.attribute(&AXAttribute::identifier()) {
            if identifier.to_string().as_str() == "Folding Ribbon Region" {
                return get_uielement_frame(&child);
            }
        }
    }

    Err(XcodeError::UIElementNotFound)
}

pub fn get_minimal_viewport_properties(
    get_via: &GetVia,
) -> Result<(ViewportProperties, CodeDocumentFrameProperties), XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;
    let code_doc_frame = get_uielement_frame(&textarea_uielement)?;

    let viewport = LAST_KNOWN_VIEWPORT_FRAME
        .lock()
        .ok_or(XcodeError::GenericError(anyhow!(
            "Viewport frame not known."
        )))?;

    Ok((
        ViewportProperties {
            dimensions: viewport.1,
            annotation_section: None,
            code_section: None,
            window_uid: viewport.0,
        },
        CodeDocumentFrameProperties {
            dimensions: code_doc_frame,
            text_offset: None,
        },
    ))
}
