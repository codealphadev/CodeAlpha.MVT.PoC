use serde::{Deserialize, Serialize};

pub use super::super::models::{AppInfo, XCodeEditorContent};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Response {
    None,
}
