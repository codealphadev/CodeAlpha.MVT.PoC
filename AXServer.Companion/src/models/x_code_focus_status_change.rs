use serde::de;
use serde::{Deserialize, Deserializer};

pub enum XCodeFocusElement {
    Editor(String),
    App(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XCodeFocusStatusChange {
    pub focus_element_change: XCodeFocusElement,
    pub is_in_focus: bool,
}
