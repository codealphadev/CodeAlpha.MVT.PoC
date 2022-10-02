use std::sync::Arc;

use parking_lot::Mutex;

use crate::window_controls::{models::TrackingAreaClickedOutsideMessage, windows::ExplainWindow};

pub fn on_click_outside_tracking_area(
    explain_window: &Arc<Mutex<ExplainWindow>>,
    outside_click_msg: &TrackingAreaClickedOutsideMessage,
) -> Option<()> {
    let mut explain_window = explain_window.try_lock()?;

    explain_window.clicked_outside(&outside_click_msg);

    Some(())
}
