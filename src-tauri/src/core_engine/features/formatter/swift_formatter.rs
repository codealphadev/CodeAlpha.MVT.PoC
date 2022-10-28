use std::path::PathBuf;
use tracing::debug;

use crate::core_engine::features::FeatureKind;
use crate::core_engine::{format_code, SwiftFormatError};
use crate::platform::macos::models::editor::ModifierKey;
use crate::platform::macos::replace_text_content;
use crate::{
    app_handle,
    core_engine::{
        events::EventRuleExecutionState,
        features::{CoreEngineTrigger, FeatureBase, FeatureError},
        utils::XcodeText,
        CodeDocument,
    },
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

pub struct SwiftFormatter {
    is_activated: bool,
}

impl FeatureBase for SwiftFormatter {
    fn kind(&self) -> FeatureKind {
        FeatureKind::Formatter
    }

    fn compute(
        &mut self,
        code_document: CodeDocument,
        trigger: CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated {
            return Ok(());
        }

        match trigger {
            CoreEngineTrigger::OnShortcutPressed(msg) => match msg.modifier {
                ModifierKey::Cmd => match msg.key.as_str() {
                    "S" => {
                        return self.format(&code_document).map_err(|err| err.into());
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

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

    fn should_compute(_kind: &FeatureKind, trigger: &CoreEngineTrigger) -> bool {
        Self::determine_procedure(trigger)
    }

    fn requires_ai(_kind: &FeatureKind, _trigger: &CoreEngineTrigger) -> bool {
        false
    }
}

impl SwiftFormatter {
    pub fn new() -> Self {
        Self {
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
        }
    }

    fn determine_procedure(trigger: &CoreEngineTrigger) -> bool {
        match trigger {
            CoreEngineTrigger::OnShortcutPressed(msg) => {
                if msg.modifier == ModifierKey::Cmd && msg.key == "S" {
                    return true;
                } else {
                    return false;
                }
            }
            _ => false,
        }
    }

    pub fn format(&self, code_document: &CodeDocument) -> Result<(), SwiftFormatError> {
        tauri::async_runtime::spawn({
            let text_content = code_document
                .text_content()
                .ok_or(SwiftFormatError::InsufficientContextForFormat)?
                .clone();

            let selected_text_range = code_document.selected_text_range().clone();

            let file_path = code_document.file_path().clone();

            async move {
                // If we use the file path pointing to the file in the repository, swiftformat will pick up any
                // local .swiftformat file and use that configuration.
                if let Some(file_path) = file_path.clone() {
                    if let Some(extension) = PathBuf::from(&file_path).extension() {
                        if !vec![Some("swift"), Some("playground")].contains(&extension.to_str()) {
                            debug!(?extension, "Attempted to run SwiftFormat on file with extension that is not swift.");
                            return;
                        }
                    }
                }

                // 1. Format the text content file
                let formatted_content = match format_code(&text_content, &file_path).await {
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
}
