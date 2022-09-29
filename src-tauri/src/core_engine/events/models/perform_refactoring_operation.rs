use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::rules::rule_base::RuleName;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/user_interaction/")]
pub struct PerformRefactoringOperationMessage {
    pub id: uuid::Uuid,
}
