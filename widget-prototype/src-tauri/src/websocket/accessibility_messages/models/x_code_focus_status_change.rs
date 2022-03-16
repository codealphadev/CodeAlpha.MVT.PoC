use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum XCodeFocusElement {
    Editor,
    App,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XCodeFocusStatusChange {
    pub focus_element_change: XCodeFocusElement,
    pub is_in_focus: bool,
}
