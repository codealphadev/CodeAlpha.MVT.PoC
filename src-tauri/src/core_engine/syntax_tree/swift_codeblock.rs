use std::str::FromStr;

use tree_sitter::Node;

use crate::{
    ax_interaction::{derive_xcode_textarea_dimensions, get_textarea_uielement},
    core_engine::{
        ax_utils::get_bounds_of_TextRange,
        rules::{TextPosition, TextRange},
        types::MatchRectangle,
    },
    utils::geometry::{LogicalPosition, LogicalSize},
};

type Err = ();

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
    pub node: Node<'a>,
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

    pub fn get_codeblock_node_bounds(&self, pid: i32) -> Option<MatchRectangle> {
        // 1. Get textarea dimensions
        let textarea_ui_element = if let Some(elem) = get_textarea_uielement(pid) {
            elem
        } else {
            return None;
        };

        let (textarea_origin, textarea_size) =
            if let Ok((origin, size)) = derive_xcode_textarea_dimensions(&textarea_ui_element) {
                (origin, size)
            } else {
                return None;
            };

        // 2. Get codeblock dimensions
        let codeblock_first_char_pos =
            TextPosition::from_TSPoint(&self.node.start_position()).as_TextIndex(&self.text);
        let codeblock_last_char_pos =
            TextPosition::from_TSPoint(&self.node.end_position()).as_TextIndex(&self.text);

        if let (Some(first_char_pos), Some(last_char_pos)) =
            (codeblock_first_char_pos, codeblock_last_char_pos)
        {
            let first_char_bounds = get_bounds_of_TextRange(
                &TextRange {
                    index: first_char_pos,
                    length: 0,
                },
                &textarea_ui_element,
            );
            let last_char_bounds = get_bounds_of_TextRange(
                &TextRange {
                    index: last_char_pos,
                    length: 0,
                },
                &textarea_ui_element,
            );

            if let (Some(first_char_bounds), Some(last_char_bounds)) =
                (first_char_bounds, last_char_bounds)
            {
                let codeblock_bounds = MatchRectangle {
                    origin: LogicalPosition {
                        x: textarea_origin.x,
                        y: first_char_bounds.origin.y,
                    },
                    size: LogicalSize {
                        width: textarea_size.width,
                        height: last_char_bounds.origin.y - first_char_bounds.origin.y,
                    },
                };
                return Some(codeblock_bounds);
            }
        }

        None
    }

    pub fn get_height_of_single_char(&self, pid: i32) -> Option<MatchRectangle> {
        let textarea_ui_element = if let Some(elem) = get_textarea_uielement(pid) {
            elem
        } else {
            return None;
        };

        if let Some(first_char_text_pos) =
            TextPosition::from_TSPoint(&self.node.start_position()).as_TextIndex(&self.text)
        {
            return get_bounds_of_TextRange(
                &TextRange {
                    index: first_char_text_pos,
                    length: 0,
                },
                &textarea_ui_element,
            );
        }

        None
    }
}
