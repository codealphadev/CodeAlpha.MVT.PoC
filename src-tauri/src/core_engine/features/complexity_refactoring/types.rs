use super::SerializedNodeSlice;
use crate::core_engine::{EditorWindowUid, XcodeText};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use ts_rs::TS;

pub type SuggestionHash = u64;
pub type SuggestionId = uuid::Uuid;
pub type SuggestionsMap = HashMap<SuggestionId, RefactoringSuggestion>;
pub type SuggestionsPerWindow = HashMap<EditorWindowUid, SuggestionsMap>;
pub type SuggestionsArcMutex = Arc<Mutex<SuggestionsPerWindow>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    pub text: XcodeText,
    pub start_index: usize,
    pub end_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub struct FERefactoringSuggestion {
    pub state: SuggestionState,
    pub new_text_content_string: Option<String>,
    pub old_text_content_string: Option<String>,
    pub new_complexity: isize,
    pub prev_complexity: isize,
    pub start_index: usize,
    pub main_function_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub enum SuggestionState {
    New,
    Recalculating,
    Ready,
}
#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub new_text_content_string: Option<String>, // TODO: test if it works with utf 16 emojis etc
    pub old_text_content_string: Option<String>,
    pub state: SuggestionState,
    pub new_complexity: isize,
    pub prev_complexity: isize,
    pub main_function_name: Option<String>,
    pub serialized_slice: SerializedNodeSlice,
    pub start_index: Option<usize>,
}
