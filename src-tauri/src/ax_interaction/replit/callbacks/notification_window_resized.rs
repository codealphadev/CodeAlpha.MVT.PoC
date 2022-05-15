use accessibility::{AXAttribute, AXUIElement, Error};
use cocoa::appkit::CGPoint;
use core_foundation::base::{CFEqual, TCFType};
use core_graphics_types::geometry::CGSize;
use tauri::{LogicalPosition, LogicalSize};

use crate::ax_interaction::{
    models::editor::EditorWindowResizedMessage, AXEventReplit, ReplitObserverState,
};

/// Notify Tauri that an editor window has been resized
/// Method requires AXUIElement of type "AXScrollBar". Asserts if different AXUIElement is provided as argument.
pub fn notify_window_resized(
    ui_element: &AXUIElement,
    replit_observer_state: &mut ReplitObserverState,
) -> Result<(), Error> {
    let window_element = ui_element.attribute(&AXAttribute::window())?;

    // Find window_element in replit_observer_state.window_list to get id
    let mut known_window = replit_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = &mut known_window {
        // Get updated window position and size
        let pos_ax_value = window_element.attribute(&AXAttribute::position())?;
        let size_ax_value = window_element.attribute(&AXAttribute::size())?;

        let origin = pos_ax_value.get_value::<CGPoint>()?;
        let size = size_ax_value.get_value::<CGSize>()?;

        // Set editor window dimensions
        let mut resize_msg = EditorWindowResizedMessage {
            id: window.0,
            window_position: tauri::LogicalPosition {
                x: origin.x,
                y: origin.y,
            },
            window_size: tauri::LogicalSize {
                width: size.width,
                height: size.height,
            },
            textarea_position: None,
            textarea_size: None,
        };

        if "AXScrollBar" == ui_element.attribute(&AXAttribute::role())? {
            // Determine editor textarea dimensions
            // For now at least, ignore errors and still continue with control flow.
            let _ = derive_resize_parameters_from_scrollbar(&mut resize_msg, ui_element);

            // Avoid spam by checking if the editor textarea dimensions have changed
            if let (Some(old_size), Some(new_size)) = (window.2, resize_msg.textarea_size) {
                if old_size.width as i32 == new_size.width as i32
                    && old_size.height as i32 == new_size.height as i32
                {
                    // Don't publish new event because nothing has changed --> the event was likely emited by a scroll event rather than resize
                    return Ok(());
                }
            }

            let new_tuple = (window.0, window.1.clone(), resize_msg.textarea_size.clone());

            // Remove item window_list
            replit_observer_state
                .window_list
                .retain(|vec_elem| vec_elem.0 != new_tuple.0);

            replit_observer_state.window_list.push(new_tuple);
        }

        AXEventReplit::EditorWindowResized(resize_msg)
            .publish_to_tauri(&replit_observer_state.app_handle);
    }

    Ok(())
}

/// The editor field (TextArea) is children to a AXScrollArea. This AXScrollArea has four children, only one of
/// which is the AXTextArea containing the source code. In order to determine the exact dimensions
/// of AXTextArea, we need to substract the width of all other children of AXScrollArea
/// From left to right, the children are:
///   - Identifier: "Source Editor Change Gutter", role: AXGroup
///   - Identifier: "Source Editor Gutter", role: "AXGroup"
///   - Identifier: -, role "AXTextArea" <-- This is the AXTextArea containing the source code
///   - Identifier: "Source Editor Minimap", role: "AXGroup"
fn derive_resize_parameters_from_scrollbar(
    resize_msg: &mut EditorWindowResizedMessage,
    scrollbar_element: &AXUIElement,
) -> Result<(), Error> {
    let role = scrollbar_element.attribute(&AXAttribute::role())?;

    assert_eq!(role.to_string(), "AXScrollBar");

    let (position, size) = derive_textarea_dimensions(scrollbar_element)?;

    // Update EditorWindowResizedMessage
    resize_msg.textarea_position = Some(position);
    resize_msg.textarea_size = Some(size);

    Ok(())
}

pub fn derive_textarea_dimensions(
    child_element: &AXUIElement,
) -> Result<(LogicalPosition<f64>, LogicalSize<f64>), Error> {
    let scrollarea_element = child_element.attribute(&AXAttribute::parent())?;

    // Get Size and Origin of AXScrollArea
    let scrollarea_pos_ax_value = scrollarea_element.attribute(&AXAttribute::position())?;
    let scrollarea_size_ax_value = scrollarea_element.attribute(&AXAttribute::size())?;

    let scrollarea_origin = scrollarea_pos_ax_value.get_value::<CGPoint>()?;
    let scrollarea_size = scrollarea_size_ax_value.get_value::<CGSize>()?;

    // Get all children
    let mut updated_width = scrollarea_size.width;
    let mut updated_origin_x = scrollarea_origin.x;
    let children_elements = scrollarea_element.attribute(&AXAttribute::children())?;

    for child in &children_elements {
        if let Ok(identifier) = child.attribute(&AXAttribute::identifier()) {
            let identifier_list: [&str; 3] = [
                "Source Editor Change Gutter",
                "Source Editor Gutter",
                "Source Editor Minimap",
            ];

            if identifier_list.contains(&identifier.to_string().as_str()) {
                updated_width -= child
                    .attribute(&AXAttribute::size())?
                    .get_value::<CGSize>()?
                    .width;

                if identifier.to_string() != "Source Editor Minimap" {
                    updated_origin_x += child
                        .attribute(&AXAttribute::size())?
                        .get_value::<CGSize>()?
                        .width;
                }
            }
        }
    }

    // Update EditorWindowResizedMessage
    let position = tauri::LogicalPosition {
        x: updated_origin_x,
        y: scrollarea_origin.y,
    };

    let size = tauri::LogicalSize {
        width: updated_width,
        height: scrollarea_size.height,
    };

    return Ok((position, size));
}
