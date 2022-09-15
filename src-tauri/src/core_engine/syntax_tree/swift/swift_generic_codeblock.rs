use std::str::FromStr;

use tree_sitter::Node;

use crate::core_engine::{syntax_tree::Complexities, TextPosition, XcodeText};

use super::{
    swift_codeblock::{
        get_first_char_position, get_last_char_position, get_node_text, SwiftCodeBlockBase,
        SwiftCodeBlockKind, SwiftCodeBlockProps,
    },
    SwiftCodeBlock, SwiftCodeBlockError,
};

pub struct SwiftGenericCodeBlock<'a> {
    props: SwiftCodeBlockProps<'a>,
    kind: SwiftCodeBlockKind,
}

impl SwiftCodeBlockBase<'_> for SwiftGenericCodeBlock<'_> {
    fn new<'a>(
        node: Node<'a>,
        node_metadata: &'a Complexities,
        text_content: &'a XcodeText,
    ) -> Result<SwiftCodeBlock<'a>, SwiftCodeBlockError> {
        let kind = SwiftCodeBlockKind::from_str(node.kind())?;
        if kind == SwiftCodeBlockKind::Function {
            return Err(SwiftCodeBlockError::WrongCodeBlockType);
        }
        Ok(SwiftCodeBlock::Other(SwiftGenericCodeBlock {
            props: SwiftCodeBlockProps {
                text_content,
                node_metadata,
                node,
            },
            kind,
        }))
    }

    fn get_kind(&self) -> SwiftCodeBlockKind {
        self.kind
    }

    // Boilerplate
    fn as_text(&self) -> std::result::Result<XcodeText, SwiftCodeBlockError> {
        get_node_text(&self.props.node, &self.props.text_content)
    }

    fn get_first_char_position(&self) -> TextPosition {
        get_first_char_position(&self.props)
    }
    fn get_last_char_position(&self) -> TextPosition {
        get_last_char_position(&self.props)
    }
}
