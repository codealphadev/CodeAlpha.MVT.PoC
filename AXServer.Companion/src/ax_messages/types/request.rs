use serde::{Deserialize, Serialize};

pub use super::super::models::{Connect, XCodeEditorContent};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "request", content = "payload")]
pub enum Request {
    Connect(Connect),
    GetXCodeEditorContent,
    UpdateXCodeEditorContent(XCodeEditorContent),
    GetXCodeFocusStatus,
    GetAppFocusState,
    None,
}
