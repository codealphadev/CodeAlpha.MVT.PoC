use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;
use ts_rs::TS;

use crate::core_engine::{
    syntax_tree::{swift_syntax_tree::NodeMetadata, SwiftSyntaxTree},
    utils::{TextPosition, TextRange, XcodeText},
};

use super::{SwiftClass, SwiftFunction, SwiftGenericCodeBlock};

#[derive(thiserror::Error, Debug)]
pub enum SwiftCodeBlockError {
    #[error("Method called on wrong code block type.")]
    WrongCodeBlockType,
    #[error("Initialization failed because node kind is unsupported.")]
    UnsupportedCodeblock,
    #[error("No valid codeblock could be derived from the provided text range.")]
    NoValidCodeblockFound,
    #[error("Generic error.")]
    GenericError(#[source] anyhow::Error),
}

pub enum SwiftCodeBlock<'a> {
    Function(SwiftFunction<'a>),
    Class(SwiftClass<'a>),
    Other(SwiftGenericCodeBlock<'a>),
}

impl<'a> SwiftCodeBlock<'a> {
    pub fn from_text_range(
        tree: &'a SwiftSyntaxTree,
        text_range: &'a TextRange,
        text_content: &'a XcodeText,
    ) -> Result<Self, SwiftCodeBlockError> {
        let node = tree
            .get_code_node_by_text_range(text_range)
            .map_err(|_err| {
                SwiftCodeBlockError::GenericError(anyhow!(
                    "get_selected_code_node failed for range: {:?}",
                    text_range
                ))
            })?;

        get_code_block_of_node(tree, node, text_content)
    }
}

pub trait SwiftCodeBlockBase<'a> {
    fn as_text(&self) -> Result<XcodeText, SwiftCodeBlockError>;
    fn new(
        tree: &'a SwiftSyntaxTree,
        node: Node<'a>,
        node_metadata: &'a NodeMetadata,
        text_content: &'a XcodeText,
    ) -> Result<SwiftCodeBlock<'a>, SwiftCodeBlockError>;
    fn get_kind(&self) -> SwiftCodeBlockKind;
    fn get_first_char_position(&self) -> TextPosition;
    fn get_last_char_position(&self) -> TextPosition;
    fn get_parent_code_block(&self) -> Result<SwiftCodeBlock, SwiftCodeBlockError>;
}

#[derive(Clone)]
pub struct SwiftCodeBlockProps<'a> {
    pub text_content: &'a XcodeText,
    pub node: Node<'a>,
    pub node_metadata: &'a NodeMetadata,
    pub tree: &'a SwiftSyntaxTree,
}

unsafe impl<'a> Send for SwiftCodeBlockProps<'a> {}
unsafe impl<'a> Sync for SwiftCodeBlockProps<'a> {}

impl<'a> SwiftCodeBlockBase<'a> for SwiftCodeBlock<'a> {
    fn as_text(&self) -> Result<XcodeText, SwiftCodeBlockError> {
        match self {
            SwiftCodeBlock::Function(f) => f.as_text(),
            SwiftCodeBlock::Class(c) => c.as_text(),
            SwiftCodeBlock::Other(o) => o.as_text(),
        }
    }

    fn new(
        tree: &'a SwiftSyntaxTree,
        node: Node<'a>,
        node_metadata: &'a NodeMetadata,
        text_content: &'a XcodeText,
    ) -> Result<Self, SwiftCodeBlockError> {
        let kind = node.kind();
        match kind {
            "function_declaration" => SwiftFunction::new(tree, node, node_metadata, text_content),
            "class_declaration" => SwiftClass::new(tree, node, node_metadata, text_content),
            "for_statement" | "if_statement" | "else_statement" | "switch_statement"
            | "while_statement" | "do_statement" | "guard_statement" => {
                SwiftGenericCodeBlock::new(tree, node, node_metadata, text_content)
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

    fn get_parent_code_block(&self) -> Result<SwiftCodeBlock, SwiftCodeBlockError> {
        match self {
            SwiftCodeBlock::Function(f) => f.get_parent_code_block(),
            SwiftCodeBlock::Class(c) => c.get_parent_code_block(),
            SwiftCodeBlock::Other(o) => o.get_parent_code_block(),
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

pub fn get_parent_code_block<'a>(
    props: &'a SwiftCodeBlockProps,
) -> Result<SwiftCodeBlock<'a>, SwiftCodeBlockError> {
    let parent_node = props
        .node
        .parent()
        .ok_or(SwiftCodeBlockError::GenericError(anyhow!(
            "No parent node found for node: {:?}",
            props.node
        )))?;
    get_code_block_of_node(props.tree, parent_node, props.text_content)
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

fn get_code_block_of_node<'a>(
    tree: &'a SwiftSyntaxTree,
    node: Node<'a>,
    text_content: &'a XcodeText,
) -> Result<SwiftCodeBlock<'a>, SwiftCodeBlockError> {
    let mut current_node = node;
    loop {
        if let Ok(codeblock) = SwiftCodeBlock::new(
            tree,
            current_node,
            tree.get_metadata_of_node(&node).map_err(|err| {
                SwiftCodeBlockError::GenericError(anyhow!(
                    "get_code_block_of_node: get_node_metadata() failed for: {:?}, err: {:?}",
                    current_node,
                    err
                ))
            })?,
            text_content,
        ) {
            return Ok(codeblock);
        } else {
            if let Some(parent) = current_node.parent() {
                current_node = parent;
            } else {
                return Err(SwiftCodeBlockError::NoValidCodeblockFound);
            }
        }
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

// https://github.com/alex-pinkus/tree-sitter-swift/blob/main/corpus/expressions.txt
pub fn is_expression(kind: &str) -> bool {
    [
        "multiplicative_expression",
        "additive_expression",
        "nil_coalescing_expression",
        "comparison_expression",
        "conjunction_expression",
        "disjunction_expression",
        "call_expression",
        "navigation_expression", // TODO
        "bitwise_operation",
        "ternary_expression",
        "equality_expression",
        "directly_assignable_expression", // Means whatever's inside is an l-expression
        "prefix_expression",
        "tuple_expression",
        "constructor_expression",
        "selector_expression",
        "self_expression",
        "open_start_range_expression",
        "range_expression",
        "open_end_range_expression",
        "postfix_expression",
        "key_path_expression",
        "key_path_string_expression",
        "check_expression",
        "try_expression",
        "await_expression",
        "prefix_expression",
    ]
    .contains(&kind)
}

pub fn is_l_expression(kind: &str) -> bool {
    kind == "directly_assignable_expression"
}
