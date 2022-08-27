use anyhow::anyhow;
use std::str::FromStr;
use tree_sitter::Node;

use crate::core_engine::utils::{TextPosition, TextRange, XcodeText};

#[derive(thiserror::Error, Debug)]
pub enum SwiftCodeblockError {
    #[error("Initialization failed because node type is unsupported.")]
    UnsupportedCodeblock,
    #[error("Generic error.")]
    GenericError(#[source] anyhow::Error),
}

pub enum SwiftCodeBlockType {
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

impl FromStr for SwiftCodeBlockType {
    type Err = SwiftCodeblockError;

    fn from_str(input: &str) -> Result<SwiftCodeBlockType, Self::Err> {
        match input {
            "for_statement" => Ok(SwiftCodeBlockType::For),
            "if_statement" => Ok(SwiftCodeBlockType::If),
            "else_statement" => Ok(SwiftCodeBlockType::Else),
            "class_body" => Ok(SwiftCodeBlockType::Class),
            "function_declaration" => Ok(SwiftCodeBlockType::Function),
            "switch_statement" => Ok(SwiftCodeBlockType::Switch),
            "while_statement" => Ok(SwiftCodeBlockType::While),
            "do_statement" => Ok(SwiftCodeBlockType::Do),
            "guard_statement" => Ok(SwiftCodeBlockType::Guard),
            _ => Err(SwiftCodeblockError::UnsupportedCodeblock),
        }
    }
}

pub struct SwiftCodeBlock<'a> {
    pub text: XcodeText,
    pub codeblock_type: SwiftCodeBlockType,
    node: Node<'a>,
}

impl<'a> SwiftCodeBlock<'a> {
    pub fn new(node: Node<'a>, text: &XcodeText) -> Result<Self, SwiftCodeblockError> {
        let codeblock_type = SwiftCodeBlockType::from_str(&node.kind())?;

        Ok(Self {
            text: text.to_owned(),
            codeblock_type,
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
