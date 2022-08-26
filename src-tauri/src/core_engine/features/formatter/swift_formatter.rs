use std::path::Path;
use tauri::api::process::{Command, CommandEvent};

use crate::{
    app_handle,
    ax_interaction::{
        get_textarea_uielement, internal::get_uielement_frame, models::editor::ModifierKey,
        send_event_mouse_wheel, set_selected_text_range, set_textarea_content, GetVia,
    },
    core_engine::{
        events::EventRuleExecutionState,
        features::{CoreEngineTrigger, FeatureBase, FeatureError},
        rules::get_bounds_of_first_char_in_range,
        utils::XcodeText,
        CodeDocument, TextPosition, TextRange,
    },
    utils::geometry::LogicalSize,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SwiftFormatError {
    #[error("SwiftFormatError: Formatter failed.")]
    FormatFailed,
    #[error("SwiftFormatError: Formatter could not run due to missing configuration.")]
    InsufficientContextForFormat,
    #[error("SwiftFormatError: File does not exist: '{0}'")]
    FileNotExisting(String),
    #[error("SwiftFormatError: Calling SwiftFormat Sidecar failed: '{0}'")]
    SidecarFailure(String),
    #[error("SwiftFormatError: SwiftFormat failed. Empty result returned.")]
    EmptyContentResult,
    #[error("SwiftFormatError: Foreign error: '{0}'")]
    ForeignError(String),
}

pub struct SwiftFormatter<'a> {
    code_doc_ref: &'a CodeDocument<'a>,
}

impl FeatureBase for SwiftFormatter<'_> {
    fn compute(&mut self, trigger: &CoreEngineTrigger) -> Result<(), FeatureError> {
        match trigger {
            CoreEngineTrigger::OnShortcutPressed(msg) => match msg.modifier {
                ModifierKey::Cmd => match msg.key.as_str() {
                    "S" => {
                        self.format();
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn update_visualization(&mut self, _trigger: &CoreEngineTrigger) -> Result<(), FeatureError> {
        // SwiftFormatter is not running on update_visualization step.
        Ok(())
    }
}

impl<'a> SwiftFormatter<'a> {
    pub fn new(code_doc_ref: &'a CodeDocument) -> Self {
        Self { code_doc_ref }
    }

    pub fn format(&self) -> Result<(), SwiftFormatError> {
        let text_content = self
            .code_doc_ref
            .text_content()
            .as_ref()
            .ok_or(SwiftFormatError::InsufficientContextForFormat)?
            .clone();
        let text_file_path = self
            .code_doc_ref
            .file_path()
            .as_ref()
            .ok_or(SwiftFormatError::InsufficientContextForFormat)?
            .clone();
        let selected_text_range = self
            .code_doc_ref
            .selected_text_range()
            .as_ref()
            .ok_or(SwiftFormatError::InsufficientContextForFormat)?
            .clone();

        tauri::async_runtime::spawn(async move {
            // 1. Format the text content file
            let formatted_content = match Self::format_file(&text_file_path).await {
                Ok(content) => content,
                Err(err) => {
                    println!("{:?}", err);
                    return;
                }
            };

            // 2. Store the position of the selected text to scroll to after formatting
            let scroll_delta = Self::scroll_dist_after_formatting(&selected_text_range);

            // 3. Update textarea content
            match set_textarea_content(&formatted_content, &GetVia::Current)
                .map_err(|err| SwiftFormatError::ForeignError(err.to_string()))
            {
                Ok(_) => {}
                Err(err) => {
                    println!("{:?}", err);
                }
            }

            // 4. Restore cursor position
            set_selected_text_range(
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
            if let Some(scroll_delta) = scroll_delta {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                send_event_mouse_wheel(scroll_delta);
            }

            // 6. Notifiy the frontend that the file has been formatted successfully
            EventRuleExecutionState::SwiftFormatFinished().publish_to_tauri(&app_handle());
        });

        Ok(())
    }

    fn scroll_dist_after_formatting(selected_text_range: &TextRange) -> Option<LogicalSize> {
        let mut scroll_delta = None;
        if let Ok(textarea_uielement) = get_textarea_uielement(&GetVia::Current) {
            if let Ok(textarea_frame) = get_uielement_frame(&textarea_uielement) {
                if let Some(bounds_of_selected_text) =
                    get_bounds_of_first_char_in_range(&selected_text_range, &textarea_uielement)
                {
                    scroll_delta = Some(LogicalSize {
                        width: textarea_frame.origin.x - bounds_of_selected_text.origin.x,
                        height: bounds_of_selected_text.origin.y - textarea_frame.origin.y,
                    });
                }
            }
        }

        scroll_delta
    }

    async fn format_file(file_path: &String) -> Result<String, SwiftFormatError> {
        if !Path::new(file_path).exists() {
            return Err(SwiftFormatError::FileNotExisting(file_path.to_string()));
        }

        let (mut rx, _) = Command::new_sidecar("swiftformat")
            .map_err(|_| {
                SwiftFormatError::SidecarFailure(
                    "failed to create `my-sidecar` binary command".to_string(),
                )
            })?
            .args([
                file_path.to_string(),
                "--output".to_string(),
                "stdout".to_string(),
                "--quiet".to_string(),
            ])
            .spawn()
            .map_err(|err| {
                SwiftFormatError::SidecarFailure(format!("SwiftFormat invocation failed: {}", err))
            })?;

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

        println!("SwiftFormatter: Cursor ending up at the bottom of the file");

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
