use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/user_interaction/")]
pub struct SwiftFormatOnCMDSMessage {
    pub swift_format_on_cmd_s_active: bool,
}
