use serde::Deserialize;
use tracing::info;

#[derive(Deserialize, Debug)]
pub enum FeedbackFeature {
    NodeExplainer,
}

impl std::fmt::Display for FeedbackFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[tauri::command]
pub fn cmd_send_feedback(feature: FeedbackFeature, feedback: String) {
    info!(?feature, ?feedback, "Feedback on feature");
}
