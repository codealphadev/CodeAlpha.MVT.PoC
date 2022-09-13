use std::env;

use serde::{Deserialize, Serialize};
use tauri::async_runtime::block_on;
use ts_rs::TS;

use crate::core_engine::syntax_tree::SwiftCodeBlockKind;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]
pub struct FunctionParameter {
    pub name: String,
    pub explanation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]
// TODO: Add function parameters
pub struct NodeExplanation {
    pub summary: String,
    pub kind: SwiftCodeBlockKind,
    pub parameters: Option<Vec<FunctionParameter>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeExplanationRequest {
    apiKey: String,
    kind: SwiftCodeBlockKind,
    code: String,
    context: String,
}

pub fn _fetch_node_explanation(
    code: &String,
    kind: SwiftCodeBlockKind,
    context: Option<String>,
) -> Option<NodeExplanation> {
    let handle = fetch_node_explanation(code, kind, context);
    block_on(handle).ok()
}

pub async fn fetch_node_explanation(
    code: &String,
    kind: SwiftCodeBlockKind,
    context: Option<String>,
) -> Result<NodeExplanation, reqwest::Error> {
    let ctx_string = if let Some(context) = context {
        context
    } else {
        "".to_string()
    };

    let url;
    let env_url = env::var("CODEALPHA_CLOUD_BACKEND_URL");
    if env_url.is_ok() {
        url = env_url.unwrap();
    } else {
        url = "https://europe-west1-analyze-text-dev.cloudfunctions.net/analyze-code".to_string();
    }

    let req_body = NodeExplanationRequest {
        apiKey: "-RWsev7z_qgP!Qinp_8cbmwgP9jg4AQBkfz".to_string(),
        code: code.clone(),
        kind,
        context: ctx_string,
    };

    let response = reqwest::Client::new()
        .post(url)
        .json(&req_body)
        .send()
        .await?;
    let parsed_response = response.json().await?;
    Ok(parsed_response)
}

#[cfg(test)]
mod tests_node_explanation_port {

    use crate::core_engine::syntax_tree::SwiftCodeBlockKind;

    use super::_fetch_node_explanation;

    #[test]
    #[ignore]
    fn test_fetch_node_explanation() {
        let resp = _fetch_node_explanation(
            &"print(\"Hello World\")".to_string(),
            SwiftCodeBlockKind::Function,
            Some("print(\"Hello World\")".to_string()),
        );
        assert!(resp.is_some());
    }

    #[test]
    #[ignore]
    fn test_fetch_node_explanation_without_context() {
        let resp = _fetch_node_explanation(
            &"print(\"Hello World\")".to_string(),
            SwiftCodeBlockKind::Function,
            None,
        );
        assert!(resp.is_some());
    }
}
