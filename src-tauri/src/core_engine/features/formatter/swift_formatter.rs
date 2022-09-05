use std::path::Path;
use tauri::api::process::{Command, CommandEvent};
use tracing::debug;

use crate::{
    app_handle,
    core_engine::{
        events::EventRuleExecutionState,
        features::{CoreEngineTrigger, FeatureBase, FeatureError},
        utils::XcodeText,
        CodeDocument, TextPosition, TextRange,
    },
    platform::macos::{
        get_bounds_for_TextRange, get_viewport_frame, models::editor::ModifierKey,
        send_event_mouse_wheel, set_selected_text_range, set_textarea_content, GetVia,
    },
    utils::geometry::LogicalSize,
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
        let text_content = code_document
            .text_content()
            .as_ref()
            .ok_or(SwiftFormatError::InsufficientContextForFormat)?
            .clone();
        let text_file_path = code_document
            .file_path()
            .as_ref()
            .ok_or(SwiftFormatError::InsufficientContextForFormat)?
            .clone();
        let selected_text_range = code_document
            .selected_text_range()
            .as_ref()
            .ok_or(SwiftFormatError::InsufficientContextForFormat)?
            .clone();

        tauri::async_runtime::spawn(async move {
            // 1. Format the text content file
            let formatted_content = match Self::format_file(&text_file_path).await {
                Ok(content) => content,
                Err(_err) => {
                    EventRuleExecutionState::SwiftFormatFailed().publish_to_tauri(&app_handle());
                    debug!(error = ?_err, "SwiftFormatFailed");
                    return;
                }
            };

            // 2. Store the position of the selected text to scroll to after formatting
            let scroll_delta = Self::scroll_dist_after_formatting(&selected_text_range);

            // 3. Update textarea content
            match set_textarea_content(&formatted_content, &GetVia::Current)
                .map_err(|err| SwiftFormatError::GenericError(err.into()))
            {
                Ok(_) => {}
                Err(_err) => {
                    EventRuleExecutionState::SwiftFormatFailed().publish_to_tauri(&app_handle());
                    debug!(error = ?_err, "SwiftFormatFailed");
                    return;
                }
            }

            // 4. Restore cursor position
            _ = set_selected_text_range(
                &TextRange {
                    index: Self::get_adjusted_cursor_index(
                        &text_content,
                        selected_text_range.index,
                        &XcodeText::from_str(formatted_content.as_str()),
                    ),
                    length: selected_text_range.length,
                },
                &GetVia::Current,
            );

            // 5. Scroll to the same position as before the formatting
            if let Ok(scroll_delta) = scroll_delta {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                _ = send_event_mouse_wheel(scroll_delta);
            }

            // 6. Notifiy the frontend that the file has been formatted successfully
            EventRuleExecutionState::SwiftFormatFinished().publish_to_tauri(&app_handle());
            debug!("SwiftFormatFinished");
        });

        Ok(())
    }

    fn scroll_dist_after_formatting(
        selected_text_range: &TextRange,
    ) -> Result<LogicalSize, SwiftFormatError> {
        if let Ok(textarea_frame) = get_viewport_frame(&GetVia::Current)
            .map_err(|err| SwiftFormatError::GenericError(err.into()))
        {
            if let Ok(bounds_of_selected_text) = get_bounds_for_TextRange(
                &TextRange {
                    index: selected_text_range.index,
                    length: 1,
                },
                &GetVia::Current,
            ) {
                return Ok(LogicalSize {
                    width: 0.0, // No horizontal scrolling
                    height: bounds_of_selected_text.origin.y - textarea_frame.origin.y,
                });
            }
        }

        Err(SwiftFormatError::GenericError(anyhow::Error::msg(
            "Could not get first char as TextRange",
        )))
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

    fn get_adjusted_cursor_index(
        pre_formatting_content: &XcodeText,
        pre_formatting_cursor_position_index: usize,
        formatted_content: &XcodeText,
    ) -> usize {
        let mut new_index = formatted_content.len();
        if let Some(text_position) = TextPosition::from_TextIndex(
            pre_formatting_content,
            pre_formatting_cursor_position_index,
        ) {
            if let Some(text_index) =
                text_position.as_TextIndex_stay_on_line(formatted_content, true)
            {
                new_index = text_index;
            }
        }

        new_index
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
