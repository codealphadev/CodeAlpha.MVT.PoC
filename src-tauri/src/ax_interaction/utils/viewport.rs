use accessibility::AXAttribute;
use cocoa::appkit::CGPoint;
use core_graphics::geometry::CGSize;

use crate::utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize};

use super::{ax_helpers::ax_attribute, get_textarea_uielement, GetVia, XcodeError};

pub fn get_viewport_frame(get_via: GetVia) -> Result<LogicalFrame, XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;

    let scrollarea_element = ax_attribute(&textarea_uielement, AXAttribute::parent())?;

    // Get Size and Origin of AXScrollArea
    let scrollarea_pos_ax_value = ax_attribute(&scrollarea_element, AXAttribute::position())?;
    let scrollarea_size_ax_value = ax_attribute(&scrollarea_element, AXAttribute::size())?;

    if let (Ok(scrollarea_origin), Ok(scrollarea_size)) = (
        scrollarea_pos_ax_value.get_value::<CGPoint>(),
        scrollarea_size_ax_value.get_value::<CGSize>(),
    ) {
        Ok(LogicalFrame {
            origin: LogicalPosition::from_CGPoint(&scrollarea_origin),
            size: LogicalSize::from_CGSize(&scrollarea_size),
        })
    } else {
        Err(XcodeError::AXResourceNotFound)
    }
}

pub fn get_code_section_frame(get_via: GetVia) -> Result<LogicalFrame, XcodeError> {
    let viewport_frame = get_viewport_frame(get_via.clone())?;

    let textarea_uielement = get_textarea_uielement(get_via.clone())?;
    let scrollarea_element = ax_attribute(&textarea_uielement, AXAttribute::parent())?;

    // Get all children
    let children_elements = ax_attribute(&scrollarea_element, AXAttribute::children())?;

    let mut updated_width = viewport_frame.size.width;
    let mut updated_origin_x = viewport_frame.origin.x;

    let mut source_editor_gutter_size = None;
    for child in &children_elements {
        if let Ok(identifier) = child.attribute(&AXAttribute::identifier()) {
            let identifier_list: [&str; 3] = [
                "Source Editor Change Gutter",
                "Source Editor Gutter",
                "Source Editor Minimap",
            ];

            if identifier_list.contains(&identifier.to_string().as_str()) {
                updated_width -= ax_attribute(&child, AXAttribute::size())?
                    .get_value::<CGSize>()
                    .map_err(|_| XcodeError::AXResourceNotFound)?
                    .width;

                if identifier.to_string() != "Source Editor Minimap" {
                    updated_origin_x += ax_attribute(&child, AXAttribute::size())?
                        .get_value::<CGSize>()
                        .map_err(|_| XcodeError::AXResourceNotFound)?
                        .width;
                }

                if identifier.to_string() == "Source Editor Gutter" {
                    source_editor_gutter_size = Some(
                        ax_attribute(&child, AXAttribute::size())?
                            .get_value::<CGSize>()
                            .map_err(|_| XcodeError::AXResourceNotFound)?
                            .width,
                    )
                }
            }
        }
    }

    // Update EditorWindowResizedMessage
    let mut position = LogicalPosition {
        x: updated_origin_x,
        y: viewport_frame.origin.y,
    };

    let mut size = LogicalSize {
        width: updated_width,
        height: viewport_frame.size.height,
    };

    // We make the textarea a little bit bigger so our annotations have more space to draw on
    let correction_width_factor = 0.105;
    if let Some(gutter_size) = source_editor_gutter_size {
        position.x -= gutter_size * correction_width_factor;
        size.width += gutter_size * correction_width_factor;
    }

    return Ok(LogicalFrame {
        origin: position,
        size,
    });
}
