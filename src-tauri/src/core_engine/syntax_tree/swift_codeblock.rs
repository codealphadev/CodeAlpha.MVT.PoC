use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tree_sitter::Node;
use ts_rs::TS;

use crate::core_engine::utils::{TextPosition, TextRange, XcodeText};

#[derive(thiserror::Error, Debug)]
pub enum SwiftCodeblockError {
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
    type Err = SwiftCodeblockError;

    fn from_str(input: &str) -> Result<SwiftCodeBlockKind, Self::Err> {
        match input {
            "for_statement" => Ok(SwiftCodeBlockKind::For),
            "if_statement" => Ok(SwiftCodeBlockKind::If),
            "else_statement" => Ok(SwiftCodeBlockKind::Else),
            "class_body" => Ok(SwiftCodeBlockKind::Class),
            "function_declaration" => Ok(SwiftCodeBlockKind::Function),
            "switch_statement" => Ok(SwiftCodeBlockKind::Switch),
            "while_statement" => Ok(SwiftCodeBlockKind::While),
            "do_statement" => Ok(SwiftCodeBlockKind::Do),
            "guard_statement" => Ok(SwiftCodeBlockKind::Guard),
            _ => Err(SwiftCodeblockError::UnsupportedCodeblock),
        }
    }
}

pub struct SwiftCodeBlock<'a> {
    pub text: XcodeText,
    pub codeblock_kind: SwiftCodeBlockKind,
    node: Node<'a>,
}

impl<'a> SwiftCodeBlock<'a> {
    pub fn new(node: Node<'a>, text: &XcodeText) -> Result<Self, SwiftCodeblockError> {
        let codeblock_kind = SwiftCodeBlockKind::from_str(&node.kind())?;

        Ok(Self {
            text: text.to_owned(),
            codeblock_kind,
            node,
        })
    }

    pub fn get_first_char_position(&self) -> TextPosition {
        TextPosition::from_TSPoint(&self.node.start_position())
    }

    pub fn get_last_char_position(&self) -> TextPosition {
        TextPosition::from_TSPoint(&self.node.end_position())
    }

    pub fn get_codeblock_text(&self) -> Result<XcodeText, SwiftCodeblockError> {
        if let Some(code_block_range) = TextRange::from_StartEndTSPoint(
            &self.text,
            &self.node.start_position(),
            &self.node.end_position(),
        ) {
            Ok(XcodeText::from_array(
                &self.text
                    [code_block_range.index..code_block_range.index + code_block_range.length],
            ))
        } else {
            Err(SwiftCodeblockError::GenericError(anyhow!(
                "get_codeblock_text: TextRange::from_StartEndTSPoint failed for: {:?}",
                self.node
            )))
        }
    }
}
