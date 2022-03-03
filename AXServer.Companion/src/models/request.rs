use serde::de;
use serde::{Deserialize, Deserializer};

pub enum RequestType {
    GetXCodeEditorContent(String),
    UpdateXCodeEditorContent(String),
    GetXCodeFocusStatus(String),
    GetAppFocusState(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub request_type: RequestType,
}
