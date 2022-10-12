use crate::core_engine::{Lsp, SwiftLsp, SwiftLspError, TextPosition, XcodeText};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use super::complexity_refactoring::Edit;

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
    tmp_file_path: &String,
) -> Result<Vec<Edit>, SwiftLspError> {
    let compiler_args = SwiftLsp::get_compiler_args(file_path, tmp_file_path).await?;

    let payload = format!(
        "key.request: source.request.semantic.refactoring
key.actionuid: source.refactoring.kind.extract.function
key.sourcefile: \"{}\"
key.line: {}
key.column: {}
key.length: {}
key.compilerargs:{}",
        tmp_file_path,
        start_position.row + 1,
        start_position.column + 1,
        length,
        format_array_as_yaml(compiler_args) //.replace(file_path, tmp_file_path)
    )
    .to_string();

    let result_str = SwiftLsp::make_lsp_request(&file_path, payload.clone()).await?;

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

#[cfg(test)]
mod tests {
    use super::{refactor_function, Edit};

    use lazy_static::lazy_static;
    use parking_lot::Mutex;

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    #[cfg(test)]
    mod swift_lsp_refactoring {
        use super::{refactor_function, Edit, MTX};
        use crate::core_engine::MockLsp;
        use crate::core_engine::TextPosition;
        use crate::core_engine::XcodeText;
        #[tokio::test]
        async fn refactor_function_test() {
            let _m = MTX.lock();
            let lsp_response = r#"key.text\" : \"extractedFunc(temperature, idx, fanSpeed)\"\n        }\n      ]\n    }\n  ]\n}\n"
{
  "key.categorizededits" : [
    {
      "key.category" : "source.edit.kind.active",
      "key.edits" : [
        {
          "key.column" : 1,
          "key.endcolumn" : 1,
          "key.endline" : 5,
          "key.line" : 5,
          "key.rangesworthnote" : [
            {
              "key.column" : 18,
              "key.endcolumn" : 31,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.basename",
              "key.line" : 1
            },
            {
              "key.argindex" : 0,
              "key.column" : 32,
              "key.endcolumn" : 33,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.decl-argument-label",
              "key.line" : 1
            },
            {
              "key.argindex" : 0,
              "key.column" : 33,
              "key.endcolumn" : 45,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.parameter-and-whitespace",
              "key.line" : 1
            },
            {
              "key.argindex" : 1,
              "key.column" : 53,
              "key.endcolumn" : 54,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.decl-argument-label",
              "key.line" : 1
            },
            {
              "key.argindex" : 1,
              "key.column" : 54,
              "key.endcolumn" : 58,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.parameter-and-whitespace",
              "key.line" : 1
            },
            {
              "key.argindex" : 2,
              "key.column" : 65,
              "key.endcolumn" : 66,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.decl-argument-label",
              "key.line" : 1
            },
            {
              "key.argindex" : 2,
              "key.column" : 66,
              "key.endcolumn" : 75,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.parameter-and-whitespace",
              "key.line" : 1
            }
          ],
          "key.text" : "fileprivate func extractedFunc(_ temperature: Int?, _ idx: Int, _ fanSpeed: Int?) {
if let value = temperature {
            gpus.list[idx].temperature = Double(value)
        
        }
        if let value = fanSpeed {
            gpus.list[idx].fanSpeed = value
        }
}

"
        }
      ]
    },
    {
      "key.category" : "source.edit.kind.active",
      "key.edits" : [
        {
          "key.column" : 9,
          "key.endcolumn" : 10,
          "key.endline" : 23,
          "key.line" : 17,
          "key.rangesworthnote" : [
            {
              "key.column" : 1,
              "key.endcolumn" : 14,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.basename",
              "key.line" : 1
            },
            {
              "key.argindex" : 0,
              "key.column" : 15,
              "key.endcolumn" : 15,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.call-argument-combined",
              "key.line" : 1
            },
            {
              "key.argindex" : 1,
              "key.column" : 28,
              "key.endcolumn" : 28,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.call-argument-combined",
              "key.line" : 1
            },
            {
              "key.argindex" : 2,
              "key.column" : 33,
              "key.endcolumn" : 33,
              "key.endline" : 1,
              "key.kind" : "source.refactoring.range.kind.call-argument-combined",
              "key.line" : 1
            }
          ],
          "key.text" : "extractedFunc(temperature, idx, fanSpeed)"
        }
      ]
    }
  ]
}"#;
            MockLsp::get_compiler_args_context()
                .expect()
                .returning(|_, _| Ok(vec!["\"-j4\"".to_string(), "\"file.svelte\"".to_string()]));

            MockLsp::make_lsp_request_context()
                .expect()
                .returning(|_, _| Ok(lsp_response.to_string()));

            let text_content = XcodeText::from_str(
                r#"
var gpus: GPUs = .init()
var list: [Int] = [1, 2, 3, 4, 5]

func read() {
    let accelerators = [1, 2, 3, 4, 5]

    let idxa: Int? = 4
    let temperature: Int? = 4
    
    let fanSpeed: Int? = 4
    for (index, accelerator) in accelerators.enumerated() {
        guard let idx = idxa else {
            return
        }

        if let value = temperature {
            gpus.list[idx].temperature = Double(value)
        
        }
        if let value = fanSpeed {
            gpus.list[idx].fanSpeed = value
        }
    }
}
"#,
            );

            XcodeText::from_str("func() { let a = b + c; }");
            let expected: Vec<Edit> = vec![];
            assert_eq!(
                dbg!(
                    refactor_function(
                        &"./file.svelte".to_string(),
                        TextPosition { row: 0, column: 16 },
                        5,
                        &text_content,
                        &"../tmp.svelte".to_string()
                    )
                    .await
                )
                .unwrap(),
                expected
            );
        }
    }
}
