use std::path::Path;
use tauri::api::process::{Command, CommandEvent};

#[derive(thiserror::Error, Debug)]
pub enum SwiftLspError {
    #[error("File does not exist: '{0}'")]
    FileNotExisting(String),
    #[error("Command failed")]
    CommandFailed(),
    #[error("Something went wrong when querying Swift LSP.")]
    GenericError(#[source] anyhow::Error),
}

pub async fn get_type_for_index(file_path: &String, index: usize) -> Result<String, SwiftLspError> {
    let sdk_path_output = std::process::Command::new("xcrun")
        .arg("--show-sdk-path")
        .arg("-sdk")
        .arg("macos")
        .output()
        .map_err(|e| SwiftLspError::GenericError(e.into()))?
        .stdout;

    let sdk_path = String::from_utf8_lossy(&sdk_path_output);

    let payload = format!(
        "key.request: source.request.cursorinfo
key.name: \"{}\"
key.sourcefile: \"{}\"
key.offset: {}
key.compilerargs:
  - \"-j4\"
  - \"{}\"
  - \"-sdk\"
  - \"{}\"",
        file_path, file_path, index, file_path, sdk_path
    )
    .to_string();
    let result_str = dbg!(make_lsp_request(&file_path, payload).await)?;
    let result =
        serde_json::from_str(&result_str).map_err(|e| SwiftLspError::GenericError(e.into()))?;
    dbg!(result);
    return Ok("TODO".to_string());
}

async fn make_lsp_request(file_path: &String, payload: String) -> Result<String, SwiftLspError> {
    if !Path::new(file_path).exists() {
        return Err(SwiftLspError::FileNotExisting(file_path.to_string()));
    }

    let (mut rx, _) = Command::new_sidecar("sourcekitten")
        .map_err(|err| SwiftLspError::GenericError(err.into()))?
        .args(["request".to_string(), "--yaml".to_string(), payload])
        .spawn()
        .map_err(|err| SwiftLspError::GenericError(err.into()))?;

    let mut text_content = "".to_string();
    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(line) = event {
            text_content.push_str(&(line + "\n"));
        }
    }

    if !text_content.is_empty() {
        Ok(text_content)
    } else {
        Err(SwiftLspError::CommandFailed())
    }
}
