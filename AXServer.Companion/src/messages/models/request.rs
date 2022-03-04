use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestType {
    GetXCodeEditorContent(String),
    UpdateXCodeEditorContent(String),
    GetXCodeFocusStatus(String),
    GetAppFocusState(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub request_type: RequestType,
}
