use serde::{Deserialize, Serialize};

pub use super::super::models::{AppInfo, XCodeEditorContent};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "response", content = "payload")]

pub enum Response {
    None,
}
