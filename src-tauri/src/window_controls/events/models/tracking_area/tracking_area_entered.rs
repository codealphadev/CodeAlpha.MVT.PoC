use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{core_engine::WindowUid, window_controls::config::AppWindow};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/window_controls/")]
pub struct TrackingAreaEnteredMessage {
    pub id: uuid::Uuid,
    pub editor_window_uid: WindowUid,
    pub app_window: AppWindow,
}
