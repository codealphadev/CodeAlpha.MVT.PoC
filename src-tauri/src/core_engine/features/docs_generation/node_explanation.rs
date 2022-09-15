use std::env;

use serde::{Deserialize, Serialize};
use tauri::async_runtime::block_on;
use ts_rs::TS;

use crate::core_engine::syntax_tree::{FunctionParameter, SwiftCodeBlockKind};

use super::node_annotation::CodeBlock;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/node_explanation/")]
pub struct FunctionParameterWithExplanation {
    pub name: String,
    pub explanation: String,
    pub param_type: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParameterDto {
    pub name: String,
    pub explanation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/node_explanation/")]
pub struct NodeExplanation {
    pub summary: String,
    pub kind: SwiftCodeBlockKind,
    pub parameters: Option<Vec<FunctionParameterWithExplanation>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NodeExplanationResponse {
    pub summary: String,
    pub kind: SwiftCodeBlockKind,
    pub parameters: Option<Vec<FunctionParameterDto>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeExplanationRequest {
    apiKey: String,
    version: String,
    kind: SwiftCodeBlockKind,
    code: String,
    context: String,
    method: String,
    parameter_names: Option<Vec<String>>,
}

pub fn _fetch_node_explanation(
    codeblock: &CodeBlock,
    context: Option<String>,
) -> Option<NodeExplanation> {
    let handle = fetch_node_explanation(codeblock, context);
    block_on(handle).ok()
}

pub async fn fetch_node_explanation(
    codeblock: &CodeBlock,
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

    let codeblock_text_string = String::from_utf16_lossy(&codeblock.text);

    let req_body = NodeExplanationRequest {
        version: "v1".to_string(),
        method: "explain".to_string(),
        apiKey: "-RWsev7z_qgP!Qinp_8cbmwgP9jg4AQBkfz".to_string(),
        code: codeblock_text_string,
        kind: codeblock.kind.clone(),
        context: ctx_string,
        parameter_names: codeblock
            .func_parameters_todo
            .as_ref()
            .map(map_function_parameters_to_names),
    };

    let response = reqwest::Client::new()
        .post(url)
        .json(&req_body)
        .send()
        .await?;
    let parsed_response: NodeExplanationResponse = response.json().await?;
    Ok(map_node_explanation_response_to_node_explanation(
        parsed_response,
        &codeblock.func_parameters_todo,
    ))
}

fn map_function_parameters_to_names(params: &Vec<FunctionParameter>) -> Vec<String> {
    params.iter().map(|p| p.name.clone()).collect()
}

fn map_node_explanation_response_to_node_explanation(
    response: NodeExplanationResponse,
    function_parameters: &Option<Vec<FunctionParameter>>,
) -> NodeExplanation {
    let parameters = if let (Some(function_parameters), Some(response_parameters)) =
        (function_parameters, response.parameters)
    {
        let mut parameters_with_explanations: Vec<FunctionParameterWithExplanation> = [].to_vec();
        for param in function_parameters {
            let response_param = response_parameters.iter().find(|p| p.name == param.name);
            if let Some(response_param) = response_param {
                parameters_with_explanations.push(FunctionParameterWithExplanation {
                    name: param.name.clone(),
                    explanation: response_param.explanation.clone(),
                    param_type: param.param_type.clone(),
                });
            }
        }
        Some(parameters_with_explanations)
    } else {
        None
    };
    NodeExplanation {
        summary: response.summary,
        kind: response.kind,
        parameters: parameters,
    }
}

#[cfg(test)]
mod tests_node_explanation_port {

    use crate::core_engine::{
        features::docs_generation::node_annotation::CodeBlock, syntax_tree::SwiftCodeBlockKind,
        TextPosition, XcodeText,
    };

    use super::_fetch_node_explanation;

    #[test]
    #[ignore]
    fn test_fetch_node_explanation() {
        let resp = _fetch_node_explanation(
            &CodeBlock {
                text: XcodeText::from_str("print(\"Hello World\")"),
                name: Some("my_fun".to_string()),
                first_char_pos: TextPosition { row: 0, column: 0 },
                last_char_pos: TextPosition { row: 0, column: 0 },
                kind: SwiftCodeBlockKind::Function,
                func_complexity_todo: None,
                func_parameters_todo: None,
            },
            Some("print(\"Hello World\")".to_string()),
        );
        assert!(resp.is_some());
    }

    #[test]
    #[ignore]
    fn test_fetch_node_explanation_without_context() {
        let resp = _fetch_node_explanation(
            &CodeBlock {
                text: XcodeText::from_str("print(\"Hello World\")"),
                name: Some("my_fun".to_string()),
                func_parameters_todo: None,
                first_char_pos: TextPosition { row: 0, column: 0 },
                last_char_pos: TextPosition { row: 0, column: 0 },
                kind: SwiftCodeBlockKind::Function,
                func_complexity_todo: None,
            },
            None,
        );
        assert!(resp.is_some());
    }
}
