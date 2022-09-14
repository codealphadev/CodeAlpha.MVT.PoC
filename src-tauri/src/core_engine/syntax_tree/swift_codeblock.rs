use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tree_sitter::Node;
use ts_rs::TS;

use crate::core_engine::utils::{TextPosition, TextRange, XcodeText};

#[derive(thiserror::Error, Debug)]
pub enum SwiftCodeBlockError {
    #[error("Method called on wrong code block type.")]
    WrongCodeBlockType,
    #[error("Initialization failed because node kind is unsupported.")]
    UnsupportedCodeblock,
    #[error("Generic error.")]
    GenericError(#[source] anyhow::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/features/docs_generation/")]

pub enum SwiftCodeBlockKind {
    For,
    If,
    Else,
    Class,
    Function,
    Switch,
    While,
    Do,
    Guard,
}

impl FromStr for SwiftCodeBlockKind {
    type Err = SwiftCodeBlockError;

    fn from_str(input: &str) -> Result<SwiftCodeBlockKind, Self::Err> {
        match input {
            "for_statement" => Ok(SwiftCodeBlockKind::For),
            "if_statement" => Ok(SwiftCodeBlockKind::If),
            "else_statement" => Ok(SwiftCodeBlockKind::Else),
            "class_declaration" => Ok(SwiftCodeBlockKind::Class),
            "function_declaration" => Ok(SwiftCodeBlockKind::Function),
            "switch_statement" => Ok(SwiftCodeBlockKind::Switch),
            "while_statement" => Ok(SwiftCodeBlockKind::While),
            "do_statement" => Ok(SwiftCodeBlockKind::Do),
            "guard_statement" => Ok(SwiftCodeBlockKind::Guard),
            _ => Err(SwiftCodeBlockError::UnsupportedCodeblock),
        }
    }
}

pub struct SwiftCodeBlock<'a> {
    pub text: XcodeText,
    pub codeblock_kind: SwiftCodeBlockKind,
    node: Node<'a>,
}

impl<'a> SwiftCodeBlock<'a> {
    pub fn new(node: Node<'a>, text: &XcodeText) -> Result<Self, SwiftCodeBlockError> {
        let codeblock_kind = SwiftCodeBlockKind::from_str(&node.kind())?;

        Ok(Self {
            text: text.to_owned(),
            codeblock_kind,
            node,
        })
    }
    // TODO: Should not be in here as it is function specific: See refactoring COD-320
    pub fn get_function_parameter_names(&self) -> Result<Vec<String>, SwiftCodeBlockError> {
        if self.codeblock_kind != SwiftCodeBlockKind::Function {
            return Err(SwiftCodeBlockError::WrongCodeBlockType);
        }
        let mut cursor = self.node.walk();
        let mut result: Vec<String> = Vec::new();
        for node in self
            .node
            .named_children(&mut cursor)
            .filter(|node| node.kind() == "parameter")
        {
            let internal_name = get_internal_name_for_parameter(&node, &self.text).ok();
            if let Some(internal_name) = internal_name {
                result.push(String::from_utf16_lossy(&internal_name));
            }
        }
        Ok(result)
    }

    pub fn get_name(&self) -> Option<String> {
        let x = self.node.child_by_field_name("name")?;
        get_text_for_node(&x, &self.text)
            .ok()
            .map(|text| String::from_utf16_lossy(&text))
    }

    pub fn get_first_char_position(&self) -> TextPosition {
        TextPosition::from_TSPoint(&self.node.start_position())
    }

    pub fn get_last_char_position(&self) -> TextPosition {
        TextPosition::from_TSPoint(&self.node.end_position())
    }

    pub fn get_codeblock_text(&self) -> Result<XcodeText, SwiftCodeBlockError> {
        get_text_for_node(&self.node, &self.text)
    }
}

fn get_text_for_node(
    node: &Node,
    text_content: &XcodeText,
) -> Result<XcodeText, SwiftCodeBlockError> {
    if let Some(code_block_range) =
        TextRange::from_StartEndTSPoint(&text_content, &node.start_position(), &node.end_position())
    {
        Ok(XcodeText::from_array(
            &text_content[code_block_range.index..code_block_range.index + code_block_range.length],
        ))
    } else {
        Err(SwiftCodeBlockError::GenericError(anyhow!(
            "get_codeblock_text: TextRange::from_StartEndTSPoint failed for: {:?}",
            node
        )))
    }
}

// TODO: Refactor COD-320
fn get_internal_name_for_parameter(
    node: &Node,
    text_content: &XcodeText,
) -> Result<XcodeText, SwiftCodeBlockError> {
    let mut cursor = node.walk();
    for name_node in node.children_by_field_name("name", &mut cursor) {
        if name_node.kind() == "simple_identifier" {
            return get_text_for_node(&name_node, &text_content);
        }
    }
    return Err(SwiftCodeBlockError::GenericError(anyhow!(
        "get_internal_name_for_parameter: Could not find internal name for parameter: {:?}",
        node
    )));
}
