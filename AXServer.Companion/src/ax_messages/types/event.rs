use serde::{Deserialize, Serialize};

pub use super::super::models::{
    AppFocusState, XCodeEditorContent, XCodeFocusStatus, XCodeFocusStatusChange,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Event {
    AppFocusState(AppFocusState),
    XCodeEditorContent(XCodeEditorContent),
    XCodeFocusStatus(XCodeFocusStatus),
    XCodeFocusStatusChange(XCodeFocusStatusChange),
    None,
}
