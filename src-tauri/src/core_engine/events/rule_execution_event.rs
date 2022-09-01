use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;

use crate::utils::messaging::ChannelList;

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "bindings/rule_execution_state/")]
#[serde(tag = "event", content = "payload")]
pub enum EventRuleExecutionState {
    SwiftFormatFinished(),
    SwiftFormatFailed(),
    DocsGenerationStarted(),
    DocsGenerationFailed(),
    DocsGenerationFinished(),
}

impl EventRuleExecutionState {
    pub fn publish_to_tauri(&self, app_handle: &tauri::AppHandle) {
        let event_name = ChannelList::EventRuleExecutionState.to_string();

        // Emit to rust listeners
        app_handle.trigger_global(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );

        _ = app_handle.emit_all(
            event_name.as_str(),
            Some(serde_json::to_string(self).unwrap()),
        );
    }
}
