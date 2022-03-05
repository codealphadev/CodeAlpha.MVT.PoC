use serde::{Deserialize, Serialize};

pub use super::super::models::{Connect, XCodeEditorContent};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "request", content = "payload")]
pub enum Request {
    Connect(Connect),
    GetXCodeEditorContent(String),
    UpdateXCodeEditorContent(XCodeEditorContent),
    GetXCodeFocusStatus(String),
    GetAppFocusState(String),
    None,
}
