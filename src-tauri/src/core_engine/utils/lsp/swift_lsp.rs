use std::collections::HashMap;

use async_trait::async_trait;
use cached::proc_macro::cached;
use mockall::automock;
use parking_lot::Mutex;
use tauri::api::process::{Command, CommandChild, CommandEvent};
use tracing::error;
use uuid::Uuid;
pub struct SwiftLsp;

use super::{
    get_compiler_args_from_xcodebuild, get_hashed_pbxproj_modification_date_with_random_fallback,
    XCodebuildError,
};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SWIFT_LSP_COMMAND_QUEUE: Mutex<HashMap<String, Vec<CommandChild>>> =
        Mutex::new(HashMap::new());
}

#[automock]
#[async_trait]
pub trait Lsp {
    async fn make_lsp_request(payload: String, use_case: String) -> Result<String, SwiftLspError>;

    async fn get_compiler_args(
        source_file_path: &Option<String>,
        tmp_file_path: &str,
    ) -> Result<Vec<String>, SwiftLspError>;
}

#[async_trait]
impl Lsp for SwiftLsp {
    async fn make_lsp_request(payload: String, use_case: String) -> Result<String, SwiftLspError> {
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
        let mut stderr = "".to_string();

        let mut termination_signal = None;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(lin) => {
                    text_content.push_str(&(lin + "\n"));
                }
                CommandEvent::Stderr(lin) => {
                    stderr.push_str(&(lin + "\n"));
                }
                CommandEvent::Terminated(payload) => {
                    termination_signal = payload.signal;
                }
                CommandEvent::Error(err) => {
                    error!("Error while running sourcekitten: {}", err);
                }
                _ => todo!(),
            }
        }

        if !text_content.is_empty() {
            Ok(text_content)
        } else if termination_signal == Some(9) {
            Err(SwiftLspError::ExecutionCancelled(None))
        } else {
            Err(SwiftLspError::SourceKittenCommandFailed(payload, stderr))
        }
    }

    async fn get_compiler_args(
        source_file_path: &Option<String>,
        tmp_file_path: &str,
    ) -> Result<Vec<String>, SwiftLspError> {
        if let Some(source_file_path) = source_file_path {
            // Try to get compiler arguments from xcodebuild
            let recompute_args_hash =
                get_hashed_pbxproj_modification_date_with_random_fallback(source_file_path);

            match get_compiler_args_from_xcodebuild(
                source_file_path.to_string(),
                recompute_args_hash,
            )
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
    #[error("Execution was cancelled: '{}'", 0)]
    ExecutionCancelled(Option<Uuid>),

    #[error("Refactoring could not be carried out: '{}'", 0)]
    RefactoringNotPossible(String),

    #[error("SourceKitten command failed: '{}' with stderr: '{}'", 0, 1)]
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
