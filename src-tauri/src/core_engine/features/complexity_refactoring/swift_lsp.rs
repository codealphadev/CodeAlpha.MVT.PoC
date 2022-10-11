use crate::core_engine::{TextPosition, XcodeText};
use anyhow::anyhow;
use cached::proc_macro::cached;
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use tauri::api::process::{Command, CommandEvent};
use tracing::error;

use super::complexity_refactoring::Edit;

#[derive(thiserror::Error, Debug)]
pub enum SwiftLspError {
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

    #[error("Something went wrong when querying Swift LSP.")]
    GenericError(#[source] anyhow::Error),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct EditDto {
    #[serde(rename = "key.column")]
    column: usize,
    #[serde(rename = "key.endcolumn")]
    endcolumn: usize,
    #[serde(rename = "key.line")]
    line: usize,
    #[serde(rename = "key.endline")]
    endline: usize,
    #[serde(rename = "key.text")]
    text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CategorizedEditDto {
    #[serde(rename = "key.edits")]
    edits: Vec<EditDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RefactoringResponse {
    #[serde(rename = "key.categorizededits")]
    categorized_edits: Vec<CategorizedEditDto>,
}

fn map_edit_dto_to_edit(
    edit_dto: EditDto,
    text_content: &XcodeText,
) -> Result<Edit, SwiftLspError> {
    Ok(Edit {
        start_index: TextPosition {
            row: edit_dto.line - 1,
            column: edit_dto.column - 1,
        }
        .as_TextIndex(text_content)
        .ok_or(SwiftLspError::GenericError(anyhow!(
            "Could not get text index for position"
        )))?,
        end_index: TextPosition {
            row: edit_dto.endline - 1,
            column: edit_dto.endcolumn - 1,
        }
        .as_TextIndex(text_content)
        .ok_or(SwiftLspError::GenericError(anyhow!(
            "Could not get text index for position"
        )))?,
        text: XcodeText::from_str(&edit_dto.text),
    })
}
fn format_array_as_yaml(compiler_args: Vec<String>) -> String {
    compiler_args
        .into_iter()
        .map(|arg| format!("\n  - {}", arg))
        .collect()
}

pub async fn refactor_function(
    file_path: &String,
    start_position: TextPosition,
    length: usize,
    text_content: &XcodeText,
) -> Result<Vec<Edit>, SwiftLspError> {
    let compiler_args = get_compiler_args(file_path).await?;

    let payload = format!(
        "key.request: source.request.semantic.refactoring
key.actionuid: source.refactoring.kind.extract.function
key.sourcefile: \"{}\"
key.line: {}
key.column: {}
key.length: {}
key.compilerargs:{}",
        file_path,
        start_position.row + 1,
        start_position.column + 1,
        length,
        format_array_as_yaml(compiler_args)
    )
    .to_string();

    let result_str = make_lsp_request(&file_path, payload.clone()).await?;

    let result: RefactoringResponse =
        serde_json::from_str(&result_str).map_err(|e| SwiftLspError::GenericError(e.into()))?;

    if result.categorized_edits.len() == 0 {
        return Err(SwiftLspError::RefactoringNotPossible(payload));
    }

    let edits: Vec<Edit> = result
        .categorized_edits
        .into_iter()
        .map(|categorized_edit| categorized_edit.edits)
        .flatten()
        .map(|edit_dto| -> Result<Edit, SwiftLspError> {
            map_edit_dto_to_edit(edit_dto, text_content)
        })
        .collect::<Result<Vec<Edit>, SwiftLspError>>()?;

    return Ok(edits);
}

async fn make_lsp_request(file_path: &String, payload: String) -> Result<String, SwiftLspError> {
    if !Path::new(file_path).exists() {
        return Err(SwiftLspError::FileNotExisting(file_path.to_string()));
    }

    let (mut rx, _) = Command::new_sidecar("sourcekitten")
        .map_err(|err| SwiftLspError::GenericError(err.into()))?
        .args(["request".to_string(), "--yaml".to_string(), payload.clone()])
        .spawn()
        .map_err(|err| SwiftLspError::GenericError(err.into()))?;

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

async fn get_compiler_args(source_file_path: &str) -> Result<Vec<String>, SwiftLspError> {
    // Try to get compiler arguments from xcodebuild
    match get_compiler_args_from_xcodebuild(source_file_path).await {
        Ok(result) => return Ok(result),
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
        format!("\"{}\"", source_file_path),
        "\"-sdk\"".to_string(),
        format!("\"{}\"", sdk_path),
    ])
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

// TODO: Cache? invalidate if future command doesn't work?
async fn get_compiler_args_from_xcodebuild(
    source_file_path: &str,
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

    extract_compiler_args_from_xcodebuild_output(&xcodebuild_output_obj, source_file_path)
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
#[cached(result = true)]
fn get_path_to_xcodeproj(file_path_str: String) -> Result<String, SwiftLspError> {
    let file_path = Path::new(&file_path_str);
    if !Path::new(file_path).exists() {
        return Err(SwiftLspError::FileNotExisting(file_path_str.to_string()));
    }
    for ancestor in file_path.ancestors() {
        let folder_str = ancestor.to_string_lossy();
        let pattern = format!("{}{}", folder_str, "/*.xcodeproj");
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
