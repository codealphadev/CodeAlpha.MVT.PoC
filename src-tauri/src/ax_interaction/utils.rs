use accessibility::{AXAttribute, AXUIElement, Error};

// Method to get the focused AXUIElement's top-level window
pub fn currently_focused_app() -> Result<AXUIElement, Error> {
    let system_wide_element = AXUIElement::system_wide();
    let focused_ui_element = system_wide_element.attribute(&AXAttribute::focused_uielement())?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    focused_window.attribute(&AXAttribute::parent())
}

// A bit WIP - tiny struct to help move context info into callback functions of the observers
pub struct TauriState {
    pub handle: tauri::AppHandle,
}
