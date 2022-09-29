use crate::core_engine::{get_cloud_function_apikey, get_cloud_function_url, XcodeText};
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use tracing::error;

#[cached(result = true, size = 100)]
pub async fn generate_function_name(code: XcodeText) -> Result<String, reqwest::Error> {
    let url = get_cloud_function_url();
    let req_body = GenerateFunctionNameRequest {
        version: "v1".to_string(),
        method: "generate-name".to_string(),
        apiKey: get_cloud_function_apikey(),
        code: String::from_utf16_lossy(&code),
    };

    let response = reqwest::Client::new()
        .post(url)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| {
            error!(
                ?e,
                "Error while sending generate_function_name request to cloud backend"
            );
            e
        })?
        .json::<Response>()
        .await
        .map_err(|e| {
            error!(
                ?e,
                "Error while parsing generate_function_name response from cloud backend"
            );
            e
        })?;

    Ok(map_generate_function_name_response_to_string(response.data))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenerateFunctionNameRequest {
    apiKey: String,
    version: String,
    code: String,
    method: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Response {
    data: GenerateFunctionNameResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GenerateFunctionNameResponse {
    name_suggestion: String,
}
fn map_generate_function_name_response_to_string(response: GenerateFunctionNameResponse) -> String {
    response.name_suggestion
}
