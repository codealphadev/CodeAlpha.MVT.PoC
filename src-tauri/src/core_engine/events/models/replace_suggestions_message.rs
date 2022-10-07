use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::core_engine::features::FERefactoringSuggestion;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/refactoring/")]
pub struct ReplaceSuggestionsMessage {
    pub suggestions: HashMap<Uuid, FERefactoringSuggestion>,
}
