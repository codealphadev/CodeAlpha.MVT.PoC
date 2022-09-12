use serde::{Deserialize, Serialize};
use tauri::async_runtime::block_on;

use crate::core_engine::syntax_tree::SwiftCodeBlockType;

#[derive(Serialize, Deserialize, Debug, Clone)]
// TODO: Add function parameters
pub struct NodeExplanationResponse {
    pub summary: String,
    pub kind: SwiftCodeBlockType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeExplanationRequest {
    apiKey: String,
    kind: SwiftCodeBlockType,
    code: String,
    context: String,
}

pub fn _fetch_node_explanation(
    code: &String,
    kind: SwiftCodeBlockType,
    context: Option<String>,
) -> Option<NodeExplanationResponse> {
    let handle = fetch_node_explanation(code, kind, context);
    block_on(handle).ok()
}

pub fn get_docstring_for_explanation(explanation: &NodeExplanationResponse) -> String {
    let mut docstring = explanation.summary.replace("\n", "\n/// ");
    docstring.insert_str(0, "/// ");
    docstring
}

pub async fn fetch_node_explanation(
    code: &String,
    kind: SwiftCodeBlockType,
    context: Option<String>,
) -> Result<NodeExplanationResponse, reqwest::Error> {
    let ctx_string = if let Some(context) = context {
        context
    } else {
        "".to_string()
    };

    let req_body = NodeExplanationRequest {
        apiKey: "-RWsev7z_qgP!Qinp_8cbmwgP9jg4AQBkfz".to_string(),
        code: code.clone(),
        kind,
        context: ctx_string,
    };

    let response = reqwest::Client::new()
        .post("https://europe-west1-analyze-text-dev.cloudfunctions.net/node-explanation")
        .json(&req_body)
        .send()
        .await?;
    let parsed_response = response.json().await?;
    Ok(parsed_response)
}

#[cfg(test)]
mod tests_node_explanation_port {

    use crate::core_engine::syntax_tree::SwiftCodeBlockType;

    use super::_fetch_node_explanation;

    #[test]
    #[ignore]
    fn test_fetch_node_explanation() {
        let resp = _fetch_node_explanation(
            &"print(\"Hello World\")".to_string(),
            SwiftCodeBlockType::Function,
            Some("print(\"Hello World\")".to_string()),
        );
        assert!(resp.is_some());
    }

    #[test]
    #[ignore]
    fn test_fetch_node_explanation_without_context() {
        let resp = _fetch_node_explanation(
            &"print(\"Hello World\")".to_string(),
            SwiftCodeBlockType::Function,
            None,
        );
        assert!(resp.is_some());
    }
}
