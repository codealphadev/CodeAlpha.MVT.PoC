use std::{collections::HashMap, fs, path::Path};

use async_trait::async_trait;
use cached::proc_macro::cached;
use glob::glob;
use mockall::automock;
use parking_lot::Mutex;
use serde_json::Value;
use tauri::api::process::{Command, CommandChild, CommandEvent};
use tracing::{error, warn};
use uuid::Uuid;
pub struct SwiftLsp;
use crate::utils::calculate_hash;
use rand::Rng;

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

#[cached(result = true, size = 100)]
async fn get_compiler_args_from_xcodebuild(
    source_file_path: String,
    _hash: u64,
) -> Result<Vec<String>, SwiftLspError> {
    let path_to_xcodeproj = get_path_to_xcodeproj(source_file_path.to_string())?;

    let output = std::process::Command::new("xcodebuild")
        .arg("-project")
        .arg(path_to_xcodeproj)
        .arg("-showBuildSettingsForIndex")
        .arg("-alltargets")
        .arg("-json")
        .output()
        .map_err(|e| SwiftLspError::GenericError(e.into()))?;

    if output.stdout.is_empty() {
        return Err(SwiftLspError::CouldNotGetBuildSettingsFromXcodebuild(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let xcodebuild_output_obj: Value =
        serde_json::from_str(&stdout).map_err(|e| SwiftLspError::GenericError(e.into()))?;

    extract_compiler_args_from_xcodebuild_output(&xcodebuild_output_obj, &source_file_path)
}

fn extract_compiler_args_from_xcodebuild_output(
    xcodebuild_output: &Value,
    source_file_path: &str,
) -> Result<Vec<String>, SwiftLspError> {
    extract_compiler_args_from_xcodebuild_output_recursive(xcodebuild_output, source_file_path)?
        .ok_or(SwiftLspError::CouldNotExtractCompilerArgsForFile(
            source_file_path.to_string(),
            xcodebuild_output.clone(),
        ))
}

fn extract_compiler_args_from_xcodebuild_output_recursive(
    object: &Value,
    source_file_path: &str,
) -> Result<Option<Vec<String>>, SwiftLspError> {
    if let Value::Object(object) = object {
        for (key, value) in object.iter() {
            if key == source_file_path {
                if let Some(Value::Array(swift_ast_command_args)) =
                    value.get("swiftASTCommandArguments")
                {
                    return Ok(Some(
                        swift_ast_command_args
                            .into_iter()
                            .map(|arg| arg.to_string())
                            .collect::<Vec<String>>(),
                    ));
                } else {
                    return Err(SwiftLspError::CouldNotFindSwiftAstCommandArgsKey(
                        source_file_path.to_string(),
                        value.clone(),
                    ));
                }
            }
            if let Some(result) =
                extract_compiler_args_from_xcodebuild_output_recursive(value, source_file_path)?
            {
                return Ok(Some(result));
            }
        }
    }
    Ok(None)
}

// TODO: Cache this according to whether we are still inside the folder? Or is it okay to recompute every time?
#[cached(result = true, time = 600)]
fn get_path_to_xcodeproj(file_path_str: String) -> Result<String, SwiftLspError> {
    let file_path = Path::new(&file_path_str);
    if !Path::new(file_path).exists() {
        return Err(SwiftLspError::FileNotExisting(file_path_str.to_string()));
    }
    for ancestor in file_path.ancestors() {
        let folder_str = ancestor.to_string_lossy();

        let pattern = format!("{}/*.xcodeproj", folder_str);
        if let Some(glob_result) = glob(&pattern)
            .map_err(|_| SwiftLspError::InvalidGlobPattern(pattern))?
            .next()
        {
            let xcodeproj_path = glob_result
                .map_err(|e| SwiftLspError::GenericError(e.into()))?
                .to_string_lossy()
                .to_string();

            return Ok(xcodeproj_path.to_string());
        }
    }
    return Err(SwiftLspError::CouldNotFindXcodeprojConfig(file_path_str));
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

// The project.pbxproj file contains the files in the project. It is one option for caching the compiler args.
fn get_hashed_pbxproj_modification_date(source_file_path: &str) -> Result<u64, SwiftLspError> {
    let path_to_xcodeproj = get_path_to_xcodeproj(source_file_path.to_string())?;

    let path_to_pbxproj = Path::new(&path_to_xcodeproj).join("project.pbxproj");

    if !path_to_pbxproj.exists() {
        error!("No project.pbxproj found in .xcodeproj!");
        return Err(SwiftLspError::CouldNotFindPbxProj);
    }

    let metadata =
        fs::metadata(path_to_pbxproj).map_err(|e| SwiftLspError::GenericError(e.into()))?;
    let modified_time = metadata
        .modified()
        .map_err(|e| SwiftLspError::GenericError(e.into()))?;
    Ok(calculate_hash(&modified_time))
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

    #[error("Could not extract compiler args from xcodebuild: File key not found")]
    CouldNotExtractCompilerArgsForFile(String, Value),

    #[error(
        "Could not extract compiler args from xcodebuild: No swiftASTCommandArguments key found"
    )]
    CouldNotFindSwiftAstCommandArgsKey(String, Value),

    #[error(
        "Could not extract compiler args from xcodebuild: Invalid glob pattern for finding .xcodeproj config file"
    )]
    InvalidGlobPattern(String),

    #[error("Could not find .xcodeproj config file")]
    CouldNotFindXcodeprojConfig(String),

    #[error("Getting build settings from xcodebuild failed")]
    CouldNotGetBuildSettingsFromXcodebuild(String),

    #[error("Could not find project.pbxproj")]
    CouldNotFindPbxProj,

    #[error("Something went wrong when querying Swift LSP.")]
    GenericError(#[source] anyhow::Error),
}
