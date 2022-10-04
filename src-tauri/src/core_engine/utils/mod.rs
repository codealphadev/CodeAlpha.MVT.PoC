pub use env::*;
pub use formatting::*;
pub use text_position::*;
pub use text_range::*;
pub use xcode_text::*;

use super::features::SwiftFormatError;
mod env;
mod formatting;
mod text_position;
mod text_range;
mod xcode_text;

// The optional file_path is used for finding .swiftformat config files
pub async fn format_code(
    input: &String,
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
    let mut formatted_content = "".to_string();
    while let Some(event) = rx.recv().await {
        if let tauri::api::process::CommandEvent::Stdout(line) = event {
            formatted_content.push_str(&(line + "\n"));
        }
    }
    Ok(formatted_content)
}
