use std::path::{Path, PathBuf};
use tauri::api::process::{Command, CommandEvent};
use tracing::debug;
use tracing::error;

use crate::platform::macos::replace_text_content;
use crate::{
    app_handle,
    core_engine::{
        events::EventRuleExecutionState,
        features::{CoreEngineTrigger, FeatureBase, FeatureError},
        rules::TemporaryFileOnDisk,
        utils::XcodeText,
        CodeDocument,
    },
    platform::macos::models::editor::ModifierKey,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

#[derive(thiserror::Error, Debug)]
pub enum SwiftFormatError {
    #[error("Formatter failed.")]
    FormatFailed,
    #[error("Formatter could not run due to missing configuration.")]
    InsufficientContextForFormat,
    #[error("File does not exist: '{0}'")]
    FileNotExisting(String),
    #[error("Something went wrong when executing this SwiftFormatter.")]
    GenericError(#[source] anyhow::Error),
}

pub struct SwiftFormatter {
    is_activated: bool,
}

impl FeatureBase for SwiftFormatter {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        match trigger {
            CoreEngineTrigger::OnShortcutPressed(msg) => match msg.modifier {
                ModifierKey::Cmd => match msg.key.as_str() {
                    "S" => {
                        return self.format(code_document).map_err(|err| err.into());
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn update_visualization(
        &mut self,
        _code_document: &CodeDocument,
        _trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        // SwiftFormatter is not running on update_visualization step.
        Ok(())
    }

    fn activate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = true;
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = false;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        // Do nothing
        Ok(())
    }
}

impl SwiftFormatter {
    pub fn new() -> Self {
        Self {
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
        }
    }

    pub fn format(&self, code_document: &CodeDocument) -> Result<(), SwiftFormatError> {
        tauri::async_runtime::spawn({
            let text_content = code_document
                .text_content()
                .as_ref()
                .ok_or(SwiftFormatError::InsufficientContextForFormat)?
                .clone();

            let selected_text_range = code_document.selected_text_range().clone();

            let file_path = code_document.file_path().clone();

            async move {
                // Either get the path from the code document or copy the text content into a temp file.
                let temp_file = match Self::create_temp_file(&text_content) {
                    Ok(temp_file) => temp_file,
                    Err(err) => {
                        EventRuleExecutionState::SwiftFormatFailed()
                            .publish_to_tauri(&app_handle());
                        debug!(error = ?err, "SwiftFormatFailed");
                        return;
                    }
                };

                let mut text_file_path = temp_file.path.to_string_lossy().to_string();

                // If a valid swift file path is provided through AX api, we read the file from disk.
                // If we use the file path pointing to the file in the repository, swiftformat will pick up any
                // local .swiftformat file and use that configuration.
                if let Some(file_path) = file_path {
                    if let Some(extension) = PathBuf::from(&file_path).extension() {
                        if extension == "swift" {
                            text_file_path = file_path;
                        }
                    }
                }

                // 1. Format the text content file
                let formatted_content = match Self::format_file(&text_file_path).await {
                    Ok(content) => content,
                    Err(err) => {
                        EventRuleExecutionState::SwiftFormatFailed()
                            .publish_to_tauri(&app_handle());
                        debug!(error = ?err, "SwiftFormatFailed");
                        return;
                    }
                };

                if text_content.as_string() == formatted_content {
                    // Nothing changed: No need to update the content
                    return;
                }

                match replace_text_content(
                    &text_content,
                    &XcodeText::from_str(&formatted_content),
                    &selected_text_range,
                )
                .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        EventRuleExecutionState::SwiftFormatFailed()
                            .publish_to_tauri(&app_handle());
                        debug!(error = ?err, "SwiftFormatFailed");
                        return;
                    }
                }

                // 6. Notify the frontend that the file has been formatted successfully
                EventRuleExecutionState::SwiftFormatFinished().publish_to_tauri(&app_handle());
                debug!("SwiftFormatFinished");
            }
        });

        Ok(())
    }

    fn create_temp_file(text_content: &XcodeText) -> Result<TemporaryFileOnDisk, SwiftFormatError> {
        let file_name = "pretzl_swiftformat.swift";
        let path_buf = std::env::temp_dir().join(file_name);

        let file = TemporaryFileOnDisk::new(path_buf, text_content.as_string());
        file.write()
            .map_err(|err| SwiftFormatError::GenericError(err.into()))?;

        Ok(file)
    }

    async fn format_file(file_path: &String) -> Result<String, SwiftFormatError> {
        if !Path::new(file_path).exists() {
            return Err(SwiftFormatError::FileNotExisting(file_path.to_string()));
        }

        let (mut rx, _) = Command::new_sidecar("swiftformat")
            .map_err(|err| SwiftFormatError::GenericError(err.into()))?
            .args([
                file_path.to_string(),
                "--output".to_string(),
                "stdout".to_string(),
                "--quiet".to_string(),
            ])
            .spawn()
            .map_err(|err| SwiftFormatError::GenericError(err.into()))?;

        let mut text_content = "".to_string();
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) = event {
                text_content.push_str(&(line + "\n"));
            }
        }

        if !text_content.is_empty() {
            Ok(text_content)
        } else {
            Err(SwiftFormatError::FormatFailed)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command as StdCommand;

    use rand::Rng;
    use tauri::async_runtime::block_on;

    use super::*;

    struct FileSystemSetup {
        pub test_file_not_existing_str: String,
        pub test_folder_dir: PathBuf,
    }

    impl FileSystemSetup {
        pub fn new() -> Self {
            let mut rng = rand::thread_rng();
            let random_number: u32 = rng.gen::<u32>();
            let test_folder_dir =
                std::env::temp_dir().join(format!("test_format_swift_file-{}", random_number));
            let test_file_path = test_folder_dir.join("test_file.txt");
            let test_file_not_existing_str = test_folder_dir
                .join("test_file_not_existing.txt")
                .to_str()
                .unwrap()
                .to_string();

            // create an empty folder temp folder
            let _ = StdCommand::new("mkdir")
                .arg(test_folder_dir.clone())
                .output()
                .expect("failed to execute process");

            assert!(test_folder_dir.exists());

            // create a file in the test_folder
            let _ = StdCommand::new("touch")
                .arg("-a")
                .arg(test_file_path.clone())
                .output()
                .expect("failed to execute process");

            Self {
                test_file_not_existing_str,
                test_folder_dir,
            }
        }
    }

    impl Drop for FileSystemSetup {
        fn drop(&mut self) {
            // remove the test folder
            let _ = StdCommand::new("rm")
                .arg("-rf")
                .arg(self.test_folder_dir.clone())
                .output()
                .expect("failed to execute process");

            assert!(!self.test_folder_dir.exists());
        }
    }

    #[test]
    fn file_not_exists() {
        let test_resources = FileSystemSetup::new();

        // Format non-existing file
        let file_path = test_resources.test_file_not_existing_str.clone();
        let handle = SwiftFormatter::format_file(&file_path);
        let formatted_file = block_on(handle);
        assert!(formatted_file.is_err());
    }
}
