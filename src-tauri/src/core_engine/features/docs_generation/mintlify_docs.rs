use serde::{Deserialize, Serialize};
use tauri::async_runtime::block_on;

#[derive(Serialize, Deserialize, Debug)]
pub struct MintlifyResponse {
    docstring: Option<String>,
    feedbackId: String,
    position: String,
    preview: String,
    shouldShowFeedback: bool,
    shouldShowShare: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintlifyRequest {
    apiKey: String,
    code: String,
    context: Option<String>,
}

pub fn get_mintlify_documentation(
    code: &String,
    context: Option<String>,
) -> Option<MintlifyResponse> {
    let handle = mintlify_documentation(code, context);
    let mintlify_response: Result<MintlifyResponse, reqwest::Error> = block_on(handle);

    if let Ok(mintlify_response) = mintlify_response {
        Some(mintlify_response)
    } else {
        None
    }
}

async fn mintlify_documentation(
    code: &String,
    context: Option<String>,
) -> Result<MintlifyResponse, reqwest::Error> {
    let req_body = MintlifyRequest {
        apiKey: "-RWsev7z_qgP!Qinp_8cbmwgP9jg4AQBkfz".to_string(),
        code: code.clone(),
        context,
    };

    let response = reqwest::Client::new()
        .post("https://europe-west1-codealpha-analyze-text-dev.cloudfunctions.net/analyze-code")
        .json(&req_body)
        .send()
        .await?;
    let parsed_response = response.json().await?;
    Ok(parsed_response)
}

#[cfg(test)]
mod tests_mintlify {

    use super::get_mintlify_documentation;

    #[test]
    fn test_get_mintlify_documentation() {
        let resp = get_mintlify_documentation(
            &"print(\"Hello World\")".to_string(),
            Some("print(\"Hello World\")".to_string()),
        );
        assert!(resp.is_some());
    }
}
