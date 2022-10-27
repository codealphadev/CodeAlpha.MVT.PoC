use rdev::{simulate, EventType};
use tokio::sync::mpsc;

use crate::{core_engine::TextRange, utils::geometry::LogicalSize};

use anyhow::anyhow;
use tracing::error;

use super::{
    get_bounds_for_TextRange, get_viewport_frame, is_focused_uielement_xcode_editor_textarea,
    GetVia, XcodeError,
};

static SCROLL_INTERVAL_MS: f64 = 10.;

pub async fn scroll_with_constant_speed(
    scroll_delta: LogicalSize,
    duration: std::time::Duration,
) -> Result<(), XcodeError> {
    if is_focused_uielement_xcode_editor_textarea()? {
        // Calculate the number of scroll intervals within the duration
        let scroll_interval_count = i32::max(
            1,
            f64::round(duration.as_millis() as f64 / SCROLL_INTERVAL_MS) as i32,
        );

        let delta_rest = scroll_delta.height as i32 % scroll_interval_count;
        let intervall_delta =
            (scroll_delta.height - delta_rest as f64) / scroll_interval_count as f64;

        for i in 0..scroll_interval_count {
            let delta = if i == 0 {
                intervall_delta + delta_rest as f64
            } else {
                intervall_delta
            };

            let event_type = EventType::Wheel {
                delta_x: 0,
                delta_y: delta as i64,
            };

            match simulate(&event_type) {
                Ok(()) => {}
                Err(_) => {
                    error!(
                        "Scrolling Error; We could not send input device event to macOS {:?}",
                        event_type
                    );
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(SCROLL_INTERVAL_MS as u64)).await;
        }
    }

    Ok(())
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

static APPROX_SCROLL_DURATION_PAGE_UP_DOWN_MS: u64 = 125;

pub async fn scroll_by_one_page(
    scroll_up: bool,
    sender: mpsc::Sender<()>,
) -> Result<(), XcodeError> {
    tokio::select! {
        res = simulate_page_scroll(scroll_up) => {
            return res;
        }
        _ = sender.closed() => {
            return Err(XcodeError::GenericError(anyhow!("Scrolling was cancelled")));
        }
    }
}

async fn simulate_page_scroll(scroll_up: bool) -> Result<(), XcodeError> {
    Ok(if is_focused_uielement_xcode_editor_textarea()? {
        // https://stackoverflow.com/questions/4965730/how-do-i-scroll-to-the-top-of-a-window-using-applescript
        if scroll_up {
            _ = simulate(&EventType::KeyPress(rdev::Key::Unknown(0x74)));
        } else {
            _ = simulate(&EventType::KeyPress(rdev::Key::Unknown(0x79)));
        }

        tokio::time::sleep(std::time::Duration::from_millis(
            APPROX_SCROLL_DURATION_PAGE_UP_DOWN_MS,
        ))
        .await
    })
}
