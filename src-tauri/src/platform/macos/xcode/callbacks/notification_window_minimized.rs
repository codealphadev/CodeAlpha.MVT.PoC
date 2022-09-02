use accessibility::{AXAttribute, AXUIElement, Error};

use crate::{
    app_handle,
    platform::macos::{
        generate_axui_element_hash, models::editor::EditorWindowMinimizedMessage, AXEventXcode,
    },
};

pub fn notify_window_minimized(window_uielement: &AXUIElement) -> Result<(), Error> {
    let role = window_uielement.attribute(&AXAttribute::role())?;
    assert_eq!(role.to_string(), "AXWindow");

    let window_uid = generate_axui_element_hash(&window_uielement);

    AXEventXcode::EditorWindowMinimized(EditorWindowMinimizedMessage { window_uid })
        .publish_to_tauri(&app_handle());

    Ok(())
}
