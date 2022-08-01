use crate::core_engine::rules::TextRange;
use std::path::Path;
use tauri::{
    api::process::{Command, CommandEvent},
    async_runtime::block_on,
};

pub struct FormattedContent {
    pub content: String,
    pub selected_text_range: TextRange,
}

pub fn format_swift_file(
    file_path: String,
    selected_text_range: TextRange,
) -> Option<FormattedContent> {
    if !Path::new(&file_path).exists() {
        println!("File does not exist: {}", file_path);
        return None;
    }
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

#[cfg(test)]
mod tests_swift_formatter {
    use std::path::PathBuf;
    use std::process::Command as StdCommand;

    use rand::Rng;

    // use crate::core_engine::rules::write_text_to_file;

    use super::*;

    struct FileSystemSetup {
        // pub formatted_content: String,
        pub test_file_not_existing_str: String,
        // pub test_file_path_str: String,
        // pub test_file_path: PathBuf,
        pub test_folder_dir: PathBuf,
        // pub unformatted_content: String,
    }

    impl FileSystemSetup {
        pub fn new() -> Self {
            let mut rng = rand::thread_rng();
            let random_number: u32 = rng.gen::<u32>();
            // let unformatted_content = "print (\"hello\")".to_string();
            // let formatted_content = "print(\"hello\")".to_string();
            let test_folder_dir =
                std::env::temp_dir().join(format!("test_format_swift_file-{}", random_number));
            let test_file_path = test_folder_dir.join("test_file.txt");
            // let test_file_path_str = test_file_path.to_str().unwrap().to_string();
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
                // formatted_content,
                test_file_not_existing_str,
                // test_file_path_str,
                // test_file_path,
                test_folder_dir,
                // unformatted_content,
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
        let text_range = TextRange::new(0, 0);

        // // Format unformatted file
        // let _ = write_text_to_file(
        //     test_resources.test_file_path.clone(),
        //     &test_resources.unformatted_content,
        // );

        // let formatted_content_option = format_swift_file(
        //     test_resources.test_file_path_str.clone(),
        //     text_range.clone(),
        // );
        // assert!(formatted_content_option.is_some());
        // let formatted_content = formatted_content_option.unwrap();
        // assert_eq!(formatted_content.content, test_resources.formatted_content);
        // assert_eq!(formatted_content.selected_text_range, text_range);

        // // Format formatted file
        // let _ = write_text_to_file(
        //     test_resources.test_file_path.clone(),
        //     &test_resources.formatted_content,
        // );
        // let formatted_content = format_swift_file(
        //     test_resources.test_file_path_str.clone(),
        //     text_range.clone(),
        // );
        // assert!(formatted_content.is_some());
        // assert_eq!(
        //     formatted_content.unwrap().content,
        //     test_resources.formatted_content
        // );

        // // Format empty file
        // let _ = write_text_to_file(test_resources.test_file_path.clone(), "");
        // let formatted_content = format_swift_file(
        //     test_resources.test_file_path_str.clone(),
        //     text_range.clone(),
        // );
        // assert!(formatted_content.is_none());

        // Format non-existing file
        let formatted_content = format_swift_file(
            test_resources.test_file_not_existing_str.clone(),
            text_range.clone(),
        );
        assert!(formatted_content.is_none());
    }
}
