use std::fmt;

use serde::{Deserialize, Serialize};

pub use super::super::models::{AppInfo, XCodeEditorContent};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "response", content = "payload")]

pub enum Response {
    None,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
