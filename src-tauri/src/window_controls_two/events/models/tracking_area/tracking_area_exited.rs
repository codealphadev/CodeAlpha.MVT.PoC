use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackingAreaExitedMessage {
    pub id: uuid::Uuid,

    /// The duration in millis from when the tracking area was entered to when tracking area was exited.
    pub duration_ms: u64,
}
