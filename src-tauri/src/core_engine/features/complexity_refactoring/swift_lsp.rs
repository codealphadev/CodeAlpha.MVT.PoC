use crate::core_engine::{TextPosition, XcodeText};
use anyhow::anyhow;
use cached::proc_macro::cached;
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
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
        .map(|arg| format!("\n  - \"{}\"", arg))
        .collect()
}

pub async fn refactor_function(
    file_path: &String,
    start_position: TextPosition,
    length: usize,
    text_content: &XcodeText,
) -> Result<Vec<Edit>, SwiftLspError> {
    let compiler_args = get_compiler_args(file_path).await.expect("//TODO:DOTODO");

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

// TODO: Cache? invalidate if future command doesn't work?
async fn get_compiler_args(source_file_path: &str) -> Option<Vec<String>> {
    // TODO: TOEODOTODOTODOTDOTODOTDOOTDODO ERROR HANDLING
    let path_to_xcodeproj = get_path_to_xcodeproj(source_file_path.to_string())
        .unwrap()
        .unwrap();
    let output = std::process::Command::new("xcodebuild")
        .arg("-project")
        .arg(path_to_xcodeproj)
        .arg("-showBuildSettingsForIndex")
        .arg("-alltargets")
        .arg("-json")
        .output()
        .unwrap() // TODO: TODOTODOTODOTODO
        .stdout;

    if output.is_empty() {
        panic!("OH no!!!!!!!!!"); // TODO: TDOODOTODTODTODODTODTODTDTO
    }
    let output_str = String::from_utf8_lossy(&output);
    let output_obj: Value = serde_json::from_str(&output_str).unwrap();
    // TODO: TOTOTOTODODOODODO
    return extract_compiler_args(&output_obj, source_file_path);
}

fn extract_compiler_args(object: &Value, source_file_path: &str) -> Option<Vec<String>> {
    if let Value::Object(object) = object {
        for (key, value) in object.iter() {
            if key == source_file_path {
                if let Some(Value::Array(swift_ast_command_args)) =
                    value.get("swiftASTCommandArguments")
                {
                    return Some(
                        swift_ast_command_args
                            .into_iter()
                            .map(|arg| arg.as_str().expect("// TODO").to_string())
                            .collect::<Vec<String>>(),
                    );
                }
                panic!("oh no! the key was there but we couldmn't parse it! // TODO:");
            }
            if let Some(result) = extract_compiler_args(value, source_file_path) {
                return Some(result);
            }
        }
    }
    None
}

// TODO: Cache this according to whether we are still inside the folder? Or is it okay to recompute every time?
#[cached(result = true)]
fn get_path_to_xcodeproj(file_path_str: String) -> Result<Option<String>, SwiftLspError> {
    let file_path = Path::new(&file_path_str);
    if !Path::new(file_path).exists() {
        return Err(SwiftLspError::FileNotExisting(file_path_str.to_string()));
    }
    for ancestor in file_path.ancestors() {
        let folder_str = ancestor.to_string_lossy();
        let pattern = format!("{}{}", folder_str, "/*.xcodeproj");
        if let Some(xcodeproj_path) = glob(&pattern)
            .expect("Very bad! // TODO")
            .next()
            .map(|res| res.expect("// TODO: oh no!!").to_string_lossy().to_string())
        {
            return Ok(Some(xcodeproj_path.to_string()));
        }
    }
    return Ok(None);
}
