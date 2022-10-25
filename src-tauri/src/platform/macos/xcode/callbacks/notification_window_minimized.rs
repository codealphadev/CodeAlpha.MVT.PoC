use crate::{
    app_handle,
    platform::macos::{
        generate_axui_element_hash, models::editor::EditorWindowMinimizedMessage, AXEventXcode,
    }, utils::assert_or_error_trace,
};
use accessibility::{AXAttribute, AXUIElement, Error};

pub fn notify_window_minimized(window_uielement: &AXUIElement) -> Result<(), Error> {
    let role = window_uielement.attribute(&AXAttribute::role())?;
    assert_or_error_trace(
        role.to_string() == "AXWindow",
        &format!(
            "notify_window_minimized() called with window_uielement of type {}; expected AXWindow",
            role.to_string()
        ),
    );

    let window_uid = generate_axui_element_hash(&window_uielement);

    AXEventXcode::EditorWindowMinimized(EditorWindowMinimizedMessage { window_uid })
        .publish_to_tauri(&app_handle());

    Ok(())
}
