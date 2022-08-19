use std::path::Path;
use tauri::{
    api::process::{Command, CommandEvent},
    async_runtime::block_on,
};

use crate::core_engine::utils::XcodeText;

pub fn format_swift_file(file_path: String) -> Option<XcodeText> {
    if !Path::new(&file_path).exists() {
        println!("File does not exist: {}", file_path);
        return None;
    }
    let handle = format_file(file_path);
    let formatted_file = block_on(handle);

    if let Some(content) = formatted_file {
        Some(content.encode_utf16().collect())
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

#[cfg(test)]
mod tests_swift_formatter {
    use std::path::PathBuf;
    use std::process::Command as StdCommand;

    use rand::Rng;

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
    fn test_format_swift_file() {
        let test_resources = FileSystemSetup::new();

        // Format non-existing file
        let formatted_content =
            format_swift_file(test_resources.test_file_not_existing_str.clone());
        assert!(formatted_content.is_none());
    }
}
