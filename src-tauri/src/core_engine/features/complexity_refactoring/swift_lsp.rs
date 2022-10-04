use crate::core_engine::{TextPosition, XcodeText};
use anyhow::anyhow;
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::api::process::{Command, CommandEvent};

use super::complexity_refactoring::Edit;

#[derive(thiserror::Error, Debug)]
pub enum SwiftLspError {
    #[error("File does not exist: '{0}'")]
    FileNotExisting(String),
    #[error("Refactoring could not be carried out")]
    RefactoringNotPossible,
    #[error("Command failed")]
    CommandFailed(),
    #[error("Something went wrong when querying Swift LSP.")]
    GenericError(#[source] anyhow::Error),
    #[error("Unable to find MacOSX SDK path")]
    CouldNotFindSdk(),
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
        return Err(SwiftLspError::CouldNotFindSdk());
    }
    let sdk_path_string = String::from_utf8_lossy(&sdk_path_output);
    Ok(sdk_path_string.trim().to_string())
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
struct CategorizedEditDto {
    #[serde(rename = "key.edits")]
    edits: Vec<EditDto>,
}

#[derive(Serialize, Deserialize)]
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

pub async fn refactor_function(
    file_path: &String,
    start_position: TextPosition,
    length: usize,
    text_content: &XcodeText,
) -> Result<Vec<Edit>, SwiftLspError> {
    let sdk_path = get_macos_sdk_path().await?;

    let payload = format!(
        "key.request: source.request.semantic.refactoring
key.actionuid: source.refactoring.kind.extract.function
key.sourcefile: \"{}\"
key.line: {}
key.column: {}
key.length: {}
key.compilerargs:
  - \"-j4\"
  - \"{}\"
  - \"-sdk\"
  - \"{}\"",
        file_path,
        start_position.row + 1, // TODO check
        start_position.column + 1,
        length,
        file_path,
        sdk_path,
    )
    .to_string();

    let result_str = make_lsp_request(&file_path, payload).await?;

    let result: RefactoringResponse =
        serde_json::from_str(&result_str).map_err(|e| SwiftLspError::GenericError(e.into()))?;

    if result.categorized_edits.len() == 0 {
        return Err(SwiftLspError::RefactoringNotPossible);
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
        .args(["request".to_string(), "--yaml".to_string(), payload])
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
        Err(SwiftLspError::CommandFailed())
    }
}
