use std::fmt;

use serde::{Deserialize, Serialize};

pub use super::super::models::{Connect, XCodeEditorContent};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "request", content = "payload")]
pub enum Request {
    Connect(Connect),
    GetXCodeEditorContent,
    UpdateXCodeEditorContent(XCodeEditorContent),
    GetXCodeFocusStatus,
    GetAppFocusState,
    None,
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
