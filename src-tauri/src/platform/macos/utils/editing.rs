use crate::{
    core_engine::{TextPosition, TextRange, XcodeText},
    platform::macos::{
        scroll_dist_viewport_to_TextRange_start, scroll_with_constant_speed,
        set_selected_text_range, set_textarea_content, GetVia, XcodeError,
    },
};

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
            _ = scroll_with_constant_speed(scroll_delta, std::time::Duration::from_millis(0));
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
