use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;
use ts_rs::TS;

use crate::core_engine::{
    syntax_tree::swift_syntax_tree::NodeMetadata,
    utils::{TextPosition, TextRange, XcodeText},
};

use super::{SwiftClass, SwiftFunction, SwiftGenericCodeBlock};

#[derive(thiserror::Error, Debug)]
pub enum SwiftCodeBlockError {
    #[error("Method called on wrong code block type.")]
    WrongCodeBlockType,
    #[error("Initialization failed because node kind is unsupported.")]
    UnsupportedCodeblock,
    #[error("Generic error.")]
    GenericError(#[source] anyhow::Error),
}

pub enum SwiftCodeBlock<'a> {
    Function(SwiftFunction<'a>),
    Class(SwiftClass<'a>),
    Other(SwiftGenericCodeBlock<'a>),
}
pub trait SwiftCodeBlockBase<'a> {
    fn as_text(&self) -> Result<XcodeText, SwiftCodeBlockError>;
    fn new(
        node: Node<'a>,
        node_metadata: &'a NodeMetadata,
        text_content: &'a XcodeText,
    ) -> Result<SwiftCodeBlock<'a>, SwiftCodeBlockError>;
    fn get_kind(&self) -> SwiftCodeBlockKind;
    fn get_first_char_position(&self) -> TextPosition;
    fn get_last_char_position(&self) -> TextPosition;
}

pub struct SwiftCodeBlockProps<'a> {
    pub text_content: &'a XcodeText,
    pub node: Node<'a>,
    pub node_metadata: &'a NodeMetadata,
}

impl<'a> SwiftCodeBlockBase<'a> for SwiftCodeBlock<'a> {
    fn as_text(&self) -> Result<XcodeText, SwiftCodeBlockError> {
        match self {
            SwiftCodeBlock::Function(f) => f.as_text(),
            SwiftCodeBlock::Class(c) => c.as_text(),
            SwiftCodeBlock::Other(o) => o.as_text(),
        }
    }

    fn new(
        node: Node<'a>,
        node_metadata: &'a NodeMetadata,
        text_content: &'a XcodeText,
    ) -> Result<Self, SwiftCodeBlockError> {
        let kind = node.kind();
        match kind {
            "function_declaration" => SwiftFunction::new(node, node_metadata, text_content),
            "class_declaration" => SwiftClass::new(node, node_metadata, text_content),
            "for_statement" | "if_statement" | "else_statement" | "switch_statement"
            | "while_statement" | "do_statement" | "guard_statement" => {
                SwiftGenericCodeBlock::new(node, node_metadata, text_content)
            }
            _ => Err(SwiftCodeBlockError::UnsupportedCodeblock),
        }
    }

    fn get_first_char_position(&self) -> TextPosition {
        match self {
            SwiftCodeBlock::Function(f) => f.get_first_char_position(),
            SwiftCodeBlock::Class(c) => c.get_first_char_position(),
            SwiftCodeBlock::Other(o) => o.get_first_char_position(),
        }
    }
    fn get_last_char_position(&self) -> TextPosition {
        match self {
            SwiftCodeBlock::Function(f) => f.get_last_char_position(),
            SwiftCodeBlock::Class(c) => c.get_last_char_position(),
            SwiftCodeBlock::Other(o) => o.get_last_char_position(),
        }
    }
    fn get_kind(&self) -> SwiftCodeBlockKind {
        match self {
            SwiftCodeBlock::Function(f) => f.get_kind(),
            SwiftCodeBlock::Class(c) => c.get_kind(),
            SwiftCodeBlock::Other(o) => o.get_kind(),
        }
    }
}

// Helper functions, to avoid reimplementing in every code block
pub fn get_first_char_position(props: &SwiftCodeBlockProps) -> TextPosition {
    TextPosition::from_TSPoint(&props.node.start_position())
}

pub fn get_last_char_position(props: &SwiftCodeBlockProps) -> TextPosition {
    TextPosition::from_TSPoint(&props.node.end_position())
}

pub fn get_node_text(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Hash)]
#[ts(export, export_to = "bindings/features/node_explanation/")]
pub enum SwiftCodeBlockKind {
    Function,
    For,
    If,
    Else,
    Class,
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
            "switch_statement" => Ok(SwiftCodeBlockKind::Switch),
            "while_statement" => Ok(SwiftCodeBlockKind::While),
            "do_statement" => Ok(SwiftCodeBlockKind::Do),
            "guard_statement" => Ok(SwiftCodeBlockKind::Guard),
            "function_statement" => Ok(SwiftCodeBlockKind::Guard),
            _ => Err(SwiftCodeBlockError::UnsupportedCodeblock),
        }
    }
}
