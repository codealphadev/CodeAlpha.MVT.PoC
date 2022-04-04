use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EditorMoveEvent {
    pub origin: tauri::PhysicalPosition<i32>,
    pub size: tauri::PhysicalSize<i32>,
    pub pid: i32,
    pub is_finished_launching: bool,
}
