use crate::{
    core_engine::{TextPosition, TextRange, XcodeText},
    platform::macos::{
        get_bounds_for_TextRange, get_viewport_frame, send_event_mouse_wheel,
        set_selected_text_range, set_textarea_content, GetVia, XcodeError,
    },
    utils::geometry::{LogicalPosition, LogicalSize},
};

use super::get_minimal_viewport_properties;

pub async fn replace_text_content(
    text_content: &XcodeText,
    new_content: &XcodeText,
    selected_text_range: &Option<TextRange>,
) -> Result<(), XcodeError> {
    // Store the position of the selected text to scroll to after formatting
    let scroll_delta = match selected_text_range {
        None => None,
        Some(selected_text_range) => {
            scroll_dist_viewport_to_TextRange_start(&selected_text_range).ok()
        }
    };

    // Update textarea content
    set_textarea_content(&new_content.as_string(), &GetVia::Current)?;

    if let Some(selected_text_range) = selected_text_range {
        // Restore cursor position
        _ = set_selected_text_range(
            &TextRange {
                index: get_adjusted_cursor_index(
                    &text_content,
                    selected_text_range.index,
                    &new_content,
                ),
                length: selected_text_range.length,
            },
            &GetVia::Current,
        );

        // Scroll to the same position as before the formatting
        if let Some(scroll_delta) = scroll_delta {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            _ = send_event_mouse_wheel(scroll_delta);
        }
    }
    Ok(())
}

fn get_adjusted_cursor_index(
    pre_formatting_content: &XcodeText,
    pre_formatting_cursor_position_index: usize,
    formatted_content: &XcodeText,
) -> usize {
    let mut new_index = formatted_content.len();
    if let Some(text_position) =
        TextPosition::from_TextIndex(pre_formatting_content, pre_formatting_cursor_position_index)
    {
        if let Some(text_index) = text_position.as_TextIndex_stay_on_line(formatted_content, true) {
            new_index = text_index;
        }
    }

    new_index
}

pub fn scroll_dist_viewport_to_TextRange_start(
    selected_text_range: &TextRange,
) -> Result<LogicalSize, XcodeError> {
    if let Ok(textarea_frame) = get_viewport_frame(&GetVia::Current) {
        if let Ok(bounds_of_selected_text) = get_bounds_for_TextRange(
            &TextRange {
                index: selected_text_range.index,
                length: 1,
            },
            &GetVia::Current,
        ) {
            return Ok(LogicalSize {
                width: 0.0, // No horizontal scrolling
                height: bounds_of_selected_text.origin.y - textarea_frame.origin.y,
            });
        }
    }

    Err(XcodeError::GenericError(anyhow::Error::msg(
        "Could not get first char as TextRange",
    )))
}

pub fn scroll_dist_viewport_to_local_position(
    local_position: &LogicalPosition,
) -> Result<LogicalSize, XcodeError> {
    let (viewport_props, code_doc_props) = get_minimal_viewport_properties(&GetVia::Current)?;

    let global_position = local_position.to_global(&code_doc_props.dimensions.origin);

    let offset_viewport_origin_local_position = LogicalSize {
        width: viewport_props.dimensions.origin.x - global_position.x,
        height: viewport_props.dimensions.origin.y - global_position.y,
    };

    Ok(offset_viewport_origin_local_position)
}
