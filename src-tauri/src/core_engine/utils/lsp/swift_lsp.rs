use std::path::Path;

use async_trait::async_trait;
use cached::proc_macro::cached;
use mockall::automock;
use tauri::api::process::{Command, CommandEvent};
use tracing::{error, warn};
pub struct SwiftLsp;
use rand::Rng;

use super::{
    get_compiler_args_from_xcodebuild, get_hashed_pbxproj_modification_date, XCodebuildError,
};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SWIFT_LSP_COMMAND_QUEUE: Mutex<HashMap<String, Vec<CommandChild>>> =
        Mutex::new(HashMap::new());
}

#[automock]
#[async_trait]
pub trait Lsp {
    async fn make_lsp_request(
        file_path: &String,
        payload: String,
        use_case: String,
    ) -> Result<String, SwiftLspError>;

    async fn get_compiler_args(
        source_file_path: &str,
        tmp_file_path: &str,
    ) -> Result<Vec<String>, SwiftLspError>;
}

#[async_trait]
impl Lsp for SwiftLsp {
    async fn make_lsp_request(
        file_path: &String,
        payload: String,
        use_case: String,
    ) -> Result<String, SwiftLspError> {
        if !Path::new(file_path).exists() {
            return Err(SwiftLspError::FileNotExisting(file_path.to_string()));
        }

        let mut rx;
        {
            let cmd_child;
            (rx, cmd_child) = Command::new_sidecar("sourcekitten")
                .map_err(|err| SwiftLspError::GenericError(err.into()))?
                .args(["request".to_string(), "--yaml".to_string(), payload.clone()])
                .spawn()
                .map_err(|err| SwiftLspError::GenericError(err.into()))?;

            let mut command_queue = SWIFT_LSP_COMMAND_QUEUE.lock();
            if let Some(commands) = command_queue.get_mut(&use_case) {
                commands.push(cmd_child);
            } else {
                command_queue.insert(use_case, vec![cmd_child]);
            }
        }

        let mut text_content = "".to_string();
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) = event {
                text_content.push_str(&(line + "\n"));
            }
        }

        if !text_content.is_empty() {
            Ok(text_content)
        } else {
            Err(SwiftLspError::SourceKittenCommandFailed(
                file_path.clone(),
                payload,
            ))
        }
    }

    async fn get_compiler_args(
        source_file_path: &str,
        tmp_file_path: &str,
    ) -> Result<Vec<String>, SwiftLspError> {
        // Try to get compiler arguments from xcodebuild
        let recompute_args_hash = match get_hashed_pbxproj_modification_date(source_file_path) {
            Err(e) => {
                warn!(
                    ?e,
                    "Unable to get hash for project.pbxproj modification date"
                );
                let mut rng = rand::thread_rng();
                rng.gen::<u64>()
            }
            Ok(res) => res,
        };

        match get_compiler_args_from_xcodebuild(source_file_path.to_string(), recompute_args_hash)
            .await
        {
            Ok(mut compiler_args) => {
                if let Some(insertion_index) = compiler_args
                    .iter()
                    .position(|a| a.contains(source_file_path))
                {
                    compiler_args.insert(insertion_index, format!("\"{}\"", tmp_file_path));
                }
                return Ok(compiler_args);
            }
            Err(e) => {
                error!(
                ?e,
                ?source_file_path,
                "Failed to get compiler arguments from Xcodebuild, will fall-back to single-file mode"
            );
            }
        }

        // Fallback in case we cannot use xcodebuild; flawed because we don't know if macOS or iOS SDK needed
        let sdk_path = get_macos_sdk_path().await?;
        Ok(vec![
            "\"-j4\"".to_string(),
            format!("\"{}\"", tmp_file_path),
            "\"-sdk\"".to_string(),
            format!("\"{}\"", sdk_path),
        ])
    }
}

#[cached(result = true, time = 600)]
async fn get_macos_sdk_path() -> Result<String, SwiftLspError> {
    let sdk_path_output = std::process::Command::new("xcrun")
        .arg("--show-sdk-path")
        .arg("-sdk")
        .arg("macosx")
        .output()
        .map_err(|e| SwiftLspError::GenericError(e.into()))?
        .stdout;

    if sdk_path_output.is_empty() {
        return Err(SwiftLspError::CouldNotFindSdk);
    }
    let sdk_path_string = String::from_utf8_lossy(&sdk_path_output);
    Ok(sdk_path_string.trim().to_string())
}

#[derive(thiserror::Error, Debug)]
pub enum SwiftLspError {
    #[error("Execution was cancelled: '{0}'")]
    ExecutionCancelled(Uuid),

    #[error("File does not exist: '{0}'")]
    FileNotExisting(String),

    #[error("Refactoring could not be carried out")]
    RefactoringNotPossible(String),

    #[error("SourceKitten command failed")]
    SourceKittenCommandFailed(String, String),

    #[error("Unable to find MacOSX SDK path")]
    CouldNotFindSdk,

    #[error("Something went wrong when querying Swift LSP.")]
    GenericError(#[source] anyhow::Error),
}

impl From<XCodebuildError> for SwiftLspError {
    fn from(cause: XCodebuildError) -> Self {
        SwiftLspError::GenericError(cause.into())
    }
}
