use serde::{Deserialize, Serialize};

pub use models::{
    AppFocusState, AppInfo, XCodeEditorContent, XCodeFocusStatus, XCodeFocusStatusChange,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum AXServerEvents {
    AppFocusState(AppFocusState),
    AppInfo(AppInfo),
    XCodeEditorContent(XCodeEditorContent),
    XCodeFocusStatus(XCodeFocusStatus),
    XCodeFocusStatusChange(XCodeFocusStatusChange),
    None,
}
