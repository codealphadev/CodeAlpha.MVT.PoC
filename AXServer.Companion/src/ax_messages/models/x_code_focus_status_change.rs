use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum XCodeFocusElement {
    Editor(String),
    App(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XCodeFocusStatusChange {
    pub focus_element_change: XCodeFocusElement,
    pub is_in_focus: bool,
}
