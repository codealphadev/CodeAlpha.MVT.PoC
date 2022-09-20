use serde::{Deserialize, Serialize};
use std::env;
use textwrap::{wrap, Options};
use tracing::error;
use ts_rs::TS;

use cached::proc_macro::cached;

use crate::{
    core_engine::{
        syntax_tree::{FunctionParameter, SwiftCodeBlockKind},
        XcodeText,
    },
    NODE_EXPLANATION_CURRENT_DOCSTRING,
};

use super::node_annotation::AnnotationCodeBlock;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ExplainResponse {
    data: NodeExplanationResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeExplanationRequest {
    apiKey: String,
    version: String,
    kind: SwiftCodeBlockKind,
    code: String,
    context: Option<String>,
    method: String,
    parameter_names: Option<Vec<String>>,
}

pub async fn fetch_node_explanation(
    code_block: AnnotationCodeBlock,
) -> Result<NodeExplanation, reqwest::Error> {
    let context = if let Some(context) = code_block.context {
        Some(String::from_utf16_lossy(&context))
    } else {
        None
    };

    let result = cached_fetch_node_explanation(
        code_block.text,
        code_block.kind,
        code_block.func_parameters_todo,
        context,
    )
    .await;
    if let Ok(node_explanation) = result.as_ref() {
        let node_docstring = explanation_to_docstring(&node_explanation);
        *NODE_EXPLANATION_CURRENT_DOCSTRING.lock() = node_docstring;
    }
    result
}

#[cached(result = true, size = 100)]
async fn cached_fetch_node_explanation(
    text: XcodeText,
    kind: SwiftCodeBlockKind,
    func_parameters: Option<Vec<FunctionParameter>>,
    context: Option<String>,
) -> Result<NodeExplanation, reqwest::Error> {
    let url;
    let env_url = env::var("CODEALPHA_CLOUD_BACKEND_URL");
    if env_url.is_ok() {
        url = env_url.unwrap();
    } else {
        url = "https://europe-west1-analyze-text-dev.cloudfunctions.net/analyze-code".to_string();
    }

    let codeblock_text_string = String::from_utf16_lossy(&text);

    let req_body = NodeExplanationRequest {
        version: "v1".to_string(),
        method: "explain".to_string(),
        apiKey: "-RWsev7z_qgP!Qinp_8cbmwgP9jg4AQBkfz".to_string(),
        code: codeblock_text_string,
        kind: kind.clone(),
        context,
        parameter_names: func_parameters
            .as_ref()
            .map(map_function_parameters_to_names),
    };

    let response = reqwest::Client::new()
        .post(url)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| {
            error!(?e, "Error while sending request to cloud backend");
            e
        })?
        .json::<ExplainResponse>()
        .await
        .map_err(|e| {
            error!(?e, "Error while parsing response from cloud backend");
            e
        })?;

    let node_explanation =
        map_node_explanation_response_to_node_explanation(response.data, func_parameters.as_ref());

    Ok(node_explanation)
}

fn map_function_parameters_to_names(params: &Vec<FunctionParameter>) -> Vec<String> {
    params.iter().map(|p| p.name.clone()).collect()
}

fn map_node_explanation_response_to_node_explanation(
    response: NodeExplanationResponse,
    function_parameters: Option<&Vec<FunctionParameter>>,
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

fn explanation_to_docstring(explanation: &NodeExplanation) -> String {
    let line_length = 80;
    let mut docstring = String::new();
    docstring.push_str(&wrap_str(
        &format!("/// {}", explanation.summary),
        line_length,
        "/// ",
    ));
    if let Some(parameters) = &explanation.parameters {
        if parameters.len() > 0 {
            docstring.push_str("\n");
            for param in parameters {
                docstring.push_str(&wrap_str(
                    &format!(
                        "/// - parameter {}: `{}` {}\n",
                        param.name, param.param_type, param.explanation
                    ),
                    line_length,
                    "/// ",
                ));
            }
            docstring.pop();
        }
    }
    docstring
}

fn wrap_str(text: &str, n: usize, insert_str: &str) -> String {
    let options = Options::new(n)
        .initial_indent("")
        .subsequent_indent(insert_str);
    let lines = wrap(text, &options);
    lines.join("\n")
}

#[cfg(test)]
mod tests {

    mod map_node_explanation_response_to_node_explanation {
        use crate::core_engine::{
            features::{
                docs_generation::{
                    node_explanation::map_node_explanation_response_to_node_explanation,
                    FunctionParameterDto, FunctionParameterWithExplanation,
                    NodeExplanationResponse,
                },
                NodeExplanation,
            },
            syntax_tree::{FunctionParameter, SwiftCodeBlockKind},
        };

        #[test]
        fn no_parameters() {
            let response = NodeExplanationResponse {
                summary: "summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: None,
            };
            assert_eq!(
                map_node_explanation_response_to_node_explanation(response, None),
                NodeExplanation {
                    summary: "summary".to_string(),
                    kind: SwiftCodeBlockKind::Function,
                    parameters: None,
                }
            );
        }

        #[test]
        fn correct_parameters() {
            let response = NodeExplanationResponse {
                summary: "summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: Some(vec![
                    FunctionParameterDto {
                        name: "param1".to_string(),
                        explanation: "It's a param".to_string(),
                    },
                    FunctionParameterDto {
                        name: "param2".to_string(),
                        explanation: "Another one".to_string(),
                    },
                ]),
            };
            let input_parameters = Some(vec![
                FunctionParameter {
                    name: "param1".to_string(),
                    param_type: "Int".to_string(),
                },
                FunctionParameter {
                    name: "param2".to_string(),
                    param_type: "String".to_string(),
                },
            ]);

            assert_eq!(
                map_node_explanation_response_to_node_explanation(
                    response,
                    input_parameters.as_ref()
                ),
                NodeExplanation {
                    summary: "summary".to_string(),
                    kind: SwiftCodeBlockKind::Function,
                    parameters: Some(vec![
                        FunctionParameterWithExplanation {
                            name: "param1".to_string(),
                            explanation: "It's a param".to_string(),
                            param_type: "Int".to_string(),
                        },
                        FunctionParameterWithExplanation {
                            name: "param2".to_string(),
                            explanation: "Another one".to_string(),
                            param_type: "String".to_string(),
                        },
                    ]),
                }
            );
        }

        #[test]
        fn filters_out_wrong_parameters() {
            let response = NodeExplanationResponse {
                summary: "summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: Some(vec![
                    FunctionParameterDto {
                        name: "crazywrongparam".to_string(),
                        explanation: "{a{ADSSfci3 xc,v.je}}".to_string(),
                    },
                    FunctionParameterDto {
                        name: "param1".to_string(),
                        explanation: "It's a param".to_string(),
                    },
                    FunctionParameterDto {
                        name: "param1".to_string(),
                        explanation: "It's a param again???".to_string(),
                    },
                    FunctionParameterDto {
                        name: "crazywrongparasdfasdam".to_string(),
                        explanation: "{a{ADSSf133qrwfasdfci3 xc,v.je}}".to_string(),
                    },
                ]),
            };
            let input_parameters = Some(vec![
                FunctionParameter {
                    name: "param1".to_string(),
                    param_type: "Int".to_string(),
                },
                FunctionParameter {
                    name: "param2".to_string(),
                    param_type: "String".to_string(),
                },
            ]);

            assert_eq!(
                map_node_explanation_response_to_node_explanation(
                    response,
                    input_parameters.as_ref()
                ),
                NodeExplanation {
                    summary: "summary".to_string(),
                    kind: SwiftCodeBlockKind::Function,
                    parameters: Some(vec![FunctionParameterWithExplanation {
                        name: "param1".to_string(),
                        explanation: "It's a param".to_string(),
                        param_type: "Int".to_string(),
                    }]),
                }
            );
        }
    }

    mod wrap_str {
        use crate::core_engine::features::docs_generation::node_explanation::wrap_str;

        #[test]
        fn no_line_breaks() {
            assert_eq!(
                wrap_str(
                    "textwrap: an efficient and powerful library for wrapping text.",
                    80,
                    "///"
                ),
                "textwrap: an efficient and powerful library for wrapping text."
            );
        }

        #[test]
        fn multiple_line_break() {
            assert_eq!(
                wrap_str(
                    "textwrap: an efficient and powerful library for wrapping text.",
                    28,
                    "///"
                ),
                "textwrap: an efficient and\n///powerful library for\n///wrapping text."
            );
        }
    }

    mod explanation_to_docstring {
        use crate::core_engine::features::docs_generation::FunctionParameterWithExplanation;

        use super::super::explanation_to_docstring;
        use super::super::NodeExplanation;

        use super::super::SwiftCodeBlockKind;

        #[test]
        fn only_summary() {
            let explanation = NodeExplanation {
                summary: "This is a summary".to_string(),
                kind: SwiftCodeBlockKind::Class,
                parameters: None,
            };
            let docstring = explanation_to_docstring(&explanation);
            assert_eq!(docstring, "/// This is a summary");
        }

        #[test]
        fn function_with_two_parameters() {
            let explanation = NodeExplanation {
                summary: "This is a summary".to_string(),
                kind: SwiftCodeBlockKind::Function,
                parameters: Some(vec![
                    FunctionParameterWithExplanation {
                        name: "param1".to_string(),
                        explanation: "This is a UIElement".to_string(),
                        param_type: "UIElement".to_string(),
                    },
                    FunctionParameterWithExplanation {
                        name: "param2".to_string(),
                        explanation: "This is a string".to_string(),
                        param_type: "String".to_string(),
                    },
                ]),
            };
            let docstring = explanation_to_docstring(&explanation);
            assert_eq!(
                docstring,
                r#"/// This is a summary
/// - parameter param1: `UIElement` This is a UIElement
/// - parameter param2: `String` This is a string"#
            );
        }
    }
    mod fetch_node_explanation {

        use tauri::async_runtime::block_on;

        use crate::core_engine::{
            features::docs_generation::node_annotation::AnnotationCodeBlock,
            syntax_tree::SwiftCodeBlockKind, TextPosition, XcodeText,
        };

        use super::super::{fetch_node_explanation, NodeExplanation};

        fn _fetch_node_explanation(codeblock: AnnotationCodeBlock) -> Option<NodeExplanation> {
            let handle = fetch_node_explanation(codeblock);
            block_on(handle).ok()
        }

        #[test]
        fn with_context() {
            let resp = _fetch_node_explanation(AnnotationCodeBlock {
                text: XcodeText::from_str("print(\"Hello World\")"),
                name: Some("my_fun".to_string()),
                first_char_pos: TextPosition { row: 0, column: 0 },
                last_char_pos: TextPosition { row: 0, column: 0 },
                kind: SwiftCodeBlockKind::Function,
                func_complexity_todo: None,
                func_parameters_todo: None,
                context: Some(XcodeText::from_str("print(\"Hello World\")")),
            });
            assert!(resp.is_some());
        }

        #[test]
        fn without_context() {
            let resp = _fetch_node_explanation(AnnotationCodeBlock {
                text: XcodeText::from_str("print(\"Hello World\")"),
                name: Some("my_fun".to_string()),
                func_parameters_todo: None,
                first_char_pos: TextPosition { row: 0, column: 0 },
                last_char_pos: TextPosition { row: 0, column: 0 },
                kind: SwiftCodeBlockKind::Function,
                func_complexity_todo: None,
                context: None,
            });
            assert!(resp.is_some());
        }
    }
}
