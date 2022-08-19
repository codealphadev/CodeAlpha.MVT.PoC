use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct TrackingAreaClickedMessage {
    pub id: uuid::Uuid,

    /// The duration in millis from when the tracking area was entered to when the click occurred.
    pub duration_ms: u64,
}
