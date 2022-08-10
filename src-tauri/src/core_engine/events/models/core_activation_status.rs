use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::rules::RuleName;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/user_interaction/")]
pub struct CoreActivationStatusMessage {
    pub engine_active: bool,
    pub active_feature: Option<RuleName>,
}
