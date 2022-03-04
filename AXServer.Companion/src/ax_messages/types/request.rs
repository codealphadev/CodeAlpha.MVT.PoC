use serde::{Deserialize, Serialize};

pub use super::super::models::{Connect, XCodeEditorContent};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Request {
    Connect(Connect),
    GetXCodeEditorContent(String),
    UpdateXCodeEditorContent(XCodeEditorContent),
    GetXCodeFocusStatus(String),
    GetAppFocusState(String),
    None,
}
