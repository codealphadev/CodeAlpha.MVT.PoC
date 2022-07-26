use tauri::{
    api::process::{Command, CommandEvent},
    async_runtime::block_on,
};

use crate::core_engine::rules::TextRange;

pub struct FormattedContent {
    pub content: String,
    pub selected_text_range: TextRange,
}

pub fn get_format_swift_file(
    file_path: String,
    selected_text_range: TextRange,
) -> Option<FormattedContent> {
    let handle = format_file(file_path);
    let formatted_file = block_on(handle);

    if let Some(content) = formatted_file {
        Some(FormattedContent {
            content,
            selected_text_range,
        })
    } else {
        None
    }
}

async fn format_file(file_path: String) -> Option<String> {
    let (mut rx, _) = Command::new_sidecar("swiftformat")
        .expect("failed to create `my-sidecar` binary command")
        .args([
            file_path,
            "--output".to_string(),
            "stdout".to_string(),
            "--quiet".to_string(),
        ])
        .spawn()
        .expect("Failed to spawn sidecar");
    let mut text_content = "".to_string();
    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(line) = event {
            text_content.push_str(&(line + "\n"));
        }
    }

    if !text_content.is_empty() {
        return Some(text_content);
    }
    None
}
