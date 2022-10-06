use crate::core_engine::features::SwiftFormatError;

// The optional file_path is used for finding .swiftformat config files
pub async fn format_code(
    input: &str,
    file_path: &Option<String>,
) -> Result<String, SwiftFormatError> {
    let mut command = tauri::api::process::Command::new_sidecar("swiftformat")
        .map_err(|err| SwiftFormatError::GenericError(err.into()))?;

    // Read .swiftformat settings from file path, even though we use direct stdin input
    if let Some(file_path) = file_path {
        command = command.args(["--stdinpath".to_string(), file_path.to_string()]);
    }

    let (mut rx, mut child) = command
        .spawn()
        .map_err(|err| SwiftFormatError::GenericError(err.into()))?;

    child
        .write(input.as_bytes())
        .expect("Failed to write to swiftformat");

    drop(child);

    let mut formatted_content = String::new();

    while let Some(event) = rx.recv().await {
        if let tauri::api::process::CommandEvent::Stdout(line) = event {
            formatted_content.push_str(&(line + "\n"));
        }
    }
    if formatted_content.len() == 0 && input.len() != 0 {
        return Err(SwiftFormatError::FormatFailed);
    }
    Ok(formatted_content)
}
