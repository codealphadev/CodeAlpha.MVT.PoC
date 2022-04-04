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
    pub ui_element_x: f64,
    pub ui_element_y: f64,
    pub ui_element_w: f64,
    pub ui_element_h: f64,
}
