use accessibility::{AXAttribute, AXUIElement};
use core_foundation::array::CFArray;
use core_graphics::geometry::CGRect;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize};

use super::{
    ax_helpers::ax_attribute, get_textarea_uielement, internal::get_uielement_frame, GetVia,
    XcodeError,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/macOS_specific/xcode/")]
pub struct ViewportProperties {
    pub dimensions: LogicalFrame,
    pub annotation_section: LogicalFrame,
    pub code_section: LogicalFrame,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/macOS_specific/xcode/")]
pub struct CodeDocumentFrameProperties {
    pub dimensions: LogicalFrame,
    pub text_offset: f64,
}

pub fn get_code_document_frame_properties(
    get_via: &GetVia,
) -> Result<CodeDocumentFrameProperties, XcodeError> {
    let code_document_uielement = get_textarea_uielement(get_via)?;
    let code_document_frame = get_uielement_frame(&code_document_uielement)?;
    let text_offset = get_text_offset_px(&get_via)?;

    Ok(CodeDocumentFrameProperties {
        dimensions: code_document_frame,
        text_offset,
    })
}

pub fn get_viewport_properties(get_via: &GetVia) -> Result<ViewportProperties, XcodeError> {
    let annotation_section = get_annotation_section_frame(&get_via)?;
    let code_section = get_code_section_frame(&get_via)?;

    let diff_origin_x = code_section.origin.x - annotation_section.origin.x;

    Ok(ViewportProperties {
        dimensions: LogicalFrame {
            origin: annotation_section.origin,
            size: LogicalSize {
                width: diff_origin_x + code_section.size.width,
                height: annotation_section.size.height,
            },
        },
        annotation_section,
        code_section,
    })
}

pub fn get_viewport_frame(get_via: &GetVia) -> Result<LogicalFrame, XcodeError> {
    let viewport_properties = get_viewport_properties(get_via)?;
    Ok(viewport_properties.dimensions)
}

pub fn get_annotation_section_frame(get_via: &GetVia) -> Result<LogicalFrame, XcodeError> {
    let viewport_uielement = get_viewport_uielement(get_via)?;

    let text_offset = get_text_offset_px(get_via)?;
    let annotation_origin_x_offset = get_annotation_origin_x_offset_px(&viewport_uielement)?;

    let viewport_frame = get_uielement_frame(&viewport_uielement)?;

    Ok(LogicalFrame {
        origin: LogicalPosition {
            x: viewport_frame.origin.x + annotation_origin_x_offset,
            y: viewport_frame.origin.y,
        },
        size: LogicalSize {
            width: text_offset - annotation_origin_x_offset,
            height: viewport_frame.size.height,
        },
    })
}

pub fn get_code_section_frame(get_via: &GetVia) -> Result<LogicalFrame, XcodeError> {
    let viewport_uielement = get_viewport_uielement(get_via)?;

    let viewport_frame = get_uielement_frame(&viewport_uielement)?;

    // Get all children
    let children_elements = ax_attribute(&viewport_uielement, AXAttribute::children())?;

    let minimap_frame = if let Ok(frame) = get_viewport_minimap_frame(&children_elements) {
        frame
    } else {
        // Return zero size if no minimap is found
        LogicalFrame::new(
            LogicalPosition { x: 0., y: 0. },
            LogicalSize {
                width: 0.,
                height: 0.,
            },
        )
    };
    let gutter_frame = get_viewport_gutter_frame(&children_elements)?;
    let gutter_change_frame = get_viewport_gutter_change_frame(&children_elements)?;

    Ok(LogicalFrame {
        origin: LogicalPosition {
            x: viewport_frame.origin.x + gutter_frame.size.width + gutter_change_frame.size.width,
            y: viewport_frame.origin.y,
        },
        size: LogicalSize {
            width: viewport_frame.size.width
                - gutter_frame.size.width
                - gutter_change_frame.size.width
                - minimap_frame.size.width,
            height: viewport_frame.size.height,
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

    println!("Could not get text offset");

    Err(XcodeError::GettingTextContentFailed)
}

fn get_annotation_origin_x_offset_px(viewport_uielement: &AXUIElement) -> Result<f64, XcodeError> {
    let viewport_children = ax_attribute(&viewport_uielement, AXAttribute::children())?;

    let gutter_frame = get_viewport_gutter_frame(&viewport_children)?;
    let change_gutter_frame = get_viewport_gutter_change_frame(&viewport_children)?;

    // We make the textarea a little bit bigger so our annotations have more space to draw on
    let correction_width_factor = 0.105;

    Ok(gutter_frame.size.width + change_gutter_frame.size.width
        - (correction_width_factor * gutter_frame.size.width))
}

fn get_viewport_uielement(get_via: &GetVia) -> Result<AXUIElement, XcodeError> {
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
