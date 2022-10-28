use async_trait::async_trait;
use cached::proc_macro::cached;
use mockall::automock;
use tauri::api::process::{Command, CommandEvent};
use tokio::sync::{mpsc, oneshot};
use tracing::error;

pub struct SwiftLsp;

use crate::core_engine::features::FeatureSignal;

use super::{
    get_compiler_args_from_xcodebuild, get_hashed_pbxproj_modification_date_with_random_fallback,
    XCodebuildError,
};

#[automock]
#[async_trait]
pub trait Lsp {
    async fn make_lsp_request(
        payload: String,
        signals_sender: &mpsc::Sender<FeatureSignal>,
    ) -> Result<String, SwiftLspError>;

    async fn get_compiler_args(
        source_file_path: &Option<String>,
        tmp_file_path: &str,
    ) -> Result<Vec<String>, SwiftLspError>;
}

#[async_trait]
impl Lsp for SwiftLsp {
    async fn make_lsp_request(
        payload: String,
        signals_sender: &mpsc::Sender<FeatureSignal>,
    ) -> Result<String, SwiftLspError> {
        // We wait for a very short time in order to allow quickly subsequently scheduled calls to cancel this one
        tokio::time::sleep(std::time::Duration::from_millis(3)).await;

        let (send, recv) = oneshot::channel();

        rayon::spawn({
            let payload = payload.clone();
            let signals_sender = signals_sender.to_owned();

            move || {
                tauri::async_runtime::spawn(async move {
                    let mut rx;
                    {
                        let command = match Command::new_sidecar("sourcekitten") {
                            Ok(cmd) => cmd,
                            Err(err) => {
                                _ = send.send(Err(SwiftLspError::GenericError(err.into())));
                                return;
                            }
                        };

                        let cmd_child;
                        (rx, cmd_child) = match command
                            .args(["request".to_string(), "--yaml".to_string(), payload.clone()])
                            .spawn()
                        {
                            Ok((rx, cmd_child)) => (rx, cmd_child),
                            Err(err) => {
                                _ = send.send(Err(SwiftLspError::GenericError(err.into())));
                                return;
                            }
                        };

                        _ = signals_sender
                            .send(FeatureSignal::SwiftLspCommandSpawned(cmd_child))
                            .await;
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
                            _ => {}
                        }
                    }

                    _ = send.send(Ok((text_content, stderr, termination_signal)));
                });
            }
        });

        let (text_content, stderr, termination_signal) = match recv.await {
            Ok(text_content) => text_content,
            Err(e) => Err(SwiftLspError::GenericError(e.into())),
        }?;

        if !text_content.is_empty() {
            Ok(text_content)
        } else if termination_signal == Some(9) {
            Err(SwiftLspError::ExecutionCancelled)
        } else {
            Err(SwiftLspError::SourceKittenCommandFailed(payload, stderr))
        }
    }

    async fn get_compiler_args(
        source_file_path: &Option<String>,
        tmp_file_path: &str,
    ) -> Result<Vec<String>, SwiftLspError> {
        // Fallback in case we cannot use xcodebuild; flawed because we don't know if macOS or iOS SDK needed
        let sdk_path = get_macos_sdk_path()?;
        let fallback_args = vec![
            "\"-j4\"".to_string(),
            format!("\"{}\"", tmp_file_path),
            "\"-sdk\"".to_string(),
            format!("\"{}\"", sdk_path),
        ];

        let source_file_path = match source_file_path {
            Some(path) => path.to_owned(),
            None => return Ok(fallback_args),
        };

        // We wait for a very short time in order to allow quickly subsequently scheduled calls to cancel this one
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        let (send, recv) = oneshot::channel();

        rayon::spawn({
            let tmp_file_path = tmp_file_path.to_string();
            let source_file_path = source_file_path.clone();

            move || {
                // Try to get compiler arguments from xcodebuild
                let recompute_args_hash =
                    get_hashed_pbxproj_modification_date_with_random_fallback(&source_file_path);

                let compiler_args = match get_compiler_args_from_xcodebuild(
                    source_file_path.clone(),
                    recompute_args_hash,
                ) {
                    Ok(mut compiler_args) => {
                        if let Some(insertion_index) = compiler_args
                            .iter()
                            .position(|a| a.contains(&source_file_path))
                        {
                            compiler_args.insert(insertion_index, format!("\"{}\"", tmp_file_path));
                        }
                        Ok(compiler_args)
                    }
                    Err(e) => Err(e),
                };

                _ = send.send(compiler_args);
            }
        });

        let compiler_args = match recv.await {
            Ok(compiler_args) => compiler_args.map_err(|e| SwiftLspError::GenericError(e.into())),
            Err(e) => Err(SwiftLspError::GenericError(e.into())),
        };

        match compiler_args {
            Ok(compiler_args) => Ok(compiler_args),
            Err(e) => {
                error!(
                    ?e,
                    ?source_file_path,
                    "Failed to get compiler arguments from Xcodebuild, will fall-back to single-file mode"
                );

                Ok(fallback_args)
            }
        }

        // Err(SwiftLspError::GenericError(anyhow!("")))
    }
}

#[cached(result = true, time = 600)]
fn get_macos_sdk_path() -> Result<String, SwiftLspError> {
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
    #[error("Execution was cancelled")]
    ExecutionCancelled,

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
