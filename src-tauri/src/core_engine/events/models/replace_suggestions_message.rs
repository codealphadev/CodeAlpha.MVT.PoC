use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::{
    features::{FERefactoringSuggestion, SuggestionId},
    EditorWindowUid,
};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub struct ReplaceSuggestionsMessage {
    pub suggestions: HashMap<EditorWindowUid, HashMap<SuggestionId, FERefactoringSuggestion>>,
}
