use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XCodeEditorContent {
    pub file_extension: String,
    pub file_name: String,
    pub file_path: String,
    pub content: String,
}
