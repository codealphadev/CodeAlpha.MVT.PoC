use std::str::FromStr;

use tree_sitter::Node;

use crate::core_engine::rules::{TextPosition};

pub type Err = ();

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
    type Err = ();

    fn from_str(input: &str) -> Result<SwiftCodeBlockType, Self::Err> {
        match input {
            "for_statement" => Ok(SwiftCodeBlockType::For),
            "if_statement" => Ok(SwiftCodeBlockType::If),
            "else_statement" => Ok(SwiftCodeBlockType::Else),
            "class_body" => Ok(SwiftCodeBlockType::Class),
            "function_body" => Ok(SwiftCodeBlockType::Function),
            "switch_statement" => Ok(SwiftCodeBlockType::Switch),
            "while_statement" => Ok(SwiftCodeBlockType::While),
            "do_statement" => Ok(SwiftCodeBlockType::Do),
            "guard_statement" => Ok(SwiftCodeBlockType::Guard),
            _ => Err(()),
        }
    }
}

pub struct SwiftCodeBlock<'a> {
    pub text: String,
    pub codeblock_type: SwiftCodeBlockType,
    node: Node<'a>,
}

impl<'a> SwiftCodeBlock<'a> {
    pub fn new(node: Node<'a>, text: &String) -> Result<Self, Err> {
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

    pub fn get_codeblock_text(&self) -> Option<String> {
        if let Ok(codeblock_text) = self.node.utf8_text(self.text.as_bytes()) {
            Some(codeblock_text.to_string())
        } else {
            None
        }
    }
}
