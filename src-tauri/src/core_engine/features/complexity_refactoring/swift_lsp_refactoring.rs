use super::{
    complexity_refactoring::Edit, COMPLEXITY_REFACTORING_EXTRACT_FUNCTION_USE_CASE,
    CURRENT_COMPLEXITY_REFACTORING_EXECUTION_ID,
};
use crate::core_engine::{Lsp, SwiftLsp, SwiftLspError, TextPosition, XcodeText};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
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
    execution_id: Uuid,
) -> Result<Edit, SwiftLspError> {
    make_sure_execution_is_most_recent(execution_id)?;

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
    tmp_file_path: &String,
    execution_id: Uuid,
) -> Result<Vec<Edit>, SwiftLspError> {
    make_sure_execution_is_most_recent(execution_id)?;
    let compiler_args = SwiftLsp::get_compiler_args(file_path, tmp_file_path).await?;
    let payload = format!(
        r#"key.request: source.request.semantic.refactoring
key.actionuid: source.refactoring.kind.extract.function
key.sourcefile: "{}"
key.line: {}
key.column: {}
key.length: {}
key.compilerargs:{}"#,
        tmp_file_path,
        start_position.row + 1,
        start_position.column + 1,
        length,
        format_array_as_yaml(compiler_args)
    )
    .to_string();

    make_sure_execution_is_most_recent(execution_id)?;

    let result_str = SwiftLsp::make_lsp_request(
        &file_path,
        payload.clone(),
        COMPLEXITY_REFACTORING_EXTRACT_FUNCTION_USE_CASE.to_string(),
    )
    .await?;

    make_sure_execution_is_most_recent(execution_id)?;

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
            map_edit_dto_to_edit(edit_dto, text_content, execution_id)
        })
        .collect::<Result<Vec<Edit>, SwiftLspError>>()?;

    return Ok(edits);
}

fn make_sure_execution_is_most_recent(execution_id: Uuid) -> Result<(), SwiftLspError> {
    if *CURRENT_COMPLEXITY_REFACTORING_EXECUTION_ID.lock() != Some(execution_id) {
        return Err(SwiftLspError::ExecutionCancelled(execution_id));
    }
    Ok(())
}
