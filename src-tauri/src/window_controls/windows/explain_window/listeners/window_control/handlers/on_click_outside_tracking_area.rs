use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    platform::macos::models::input_device::ClickType,
    window_controls::{models::TrackingAreaClickedOutsideMessage, windows::ExplainWindow},
};

pub fn on_click_outside_tracking_area(
    explain_window: &Arc<Mutex<ExplainWindow>>,
    outside_click_msg: &TrackingAreaClickedOutsideMessage,
) -> Option<()> {
    let mut explain_window = explain_window.try_lock()?;

    if outside_click_msg.click_type == ClickType::Down {
        explain_window.clicked_outside(&outside_click_msg);
    }

    Some(())
}
