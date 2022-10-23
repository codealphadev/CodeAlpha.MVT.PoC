use serde::Deserialize;
use tracing::info;
use ts_rs::TS;

#[derive(Deserialize, Debug, TS)]
#[ts(export, export_to = "bindings/feedback/")]
pub enum FeedbackTarget {
    NodeExplainer,
    MethodExtraction,
}

impl std::fmt::Display for FeedbackTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[tauri::command]
pub fn cmd_send_feedback(target: FeedbackTarget, feedback: String) {
    info!(?target, ?feedback, "Feedback on feature");
}
