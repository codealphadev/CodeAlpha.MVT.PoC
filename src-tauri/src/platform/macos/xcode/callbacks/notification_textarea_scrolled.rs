use crate::{
    platform::macos::{
        models::editor::EditorTextareaScrolledMessage, xcode::XCodeObserverState, AXEventXcode,
        EventViewport, GetVia,
    },
    utils::assert_or_error_trace,
};
use accessibility::{AXUIElement, AXUIElementAttributes, Error};
use lazy_static::lazy_static;
use parking_lot::Mutex;

use std::time::Duration;

use core_foundation::base::{CFEqual, TCFType};
use throttle::Throttle;

lazy_static! {
    static ref SCROLL_THROTTLE: Mutex<Throttle> =
        Mutex::new(Throttle::new(Duration::from_millis(8), 1));
}
pub fn notify_textarea_scrolled(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    assert_or_error_trace(
        uielement.role()?.to_string() == "AXScrollBar",
        &format!(
            "notify_textarea_scrolled() called with AXUIElement of type {}; expected AXScrollBar",
            uielement.role()?.to_string()
        ),
    );
    let _ = SCROLL_THROTTLE.try_lock().ok_or(Error::NotFound)?.accept();

    let window_element = uielement.window()?;

    // Find window_element in xcode_observer_state.window_list to get id
    let mut known_window = xcode_observer_state
        .window_list
        .iter()
        .find(|&vec_elem| unsafe {
            CFEqual(window_element.as_CFTypeRef(), vec_elem.1.as_CFTypeRef()) != 0
        });

    if let Some(window) = &mut known_window {
        // Send it the updated viewport to the frontend
        update_code_document_dimensions(&xcode_observer_state.app_handle).ok();

        AXEventXcode::EditorTextareaScrolled(EditorTextareaScrolledMessage {
            window_uid: window.0,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);
    }
    Ok(())
}

pub fn update_code_document_dimensions(app_handle: &tauri::AppHandle) -> Result<(), Error> {
    // This event needs to arrive in the frontend before any annotation events
    // because the frontend relies on always having the correct document rect to handle global coordinates sent from core engine.
    EventViewport::new_xcode_viewport_update_minimal(&GetVia::Current)
        .map_err(|_| accessibility::Error::NotFound)?
        .publish_to_tauri(app_handle);

    Ok(())
}
