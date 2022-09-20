use tree_sitter::Node;

use crate::core_engine::{
    syntax_tree::{
        swift_syntax_tree::NodeMetadata, SwiftCodeBlock, SwiftCodeBlockError, SwiftSyntaxTree,
    },
    TextPosition, XcodeText,
};

use super::swift_code_block::{
    get_first_char_position, get_last_char_position, get_node_text, get_parent_code_block,
    SwiftCodeBlockBase, SwiftCodeBlockKind, SwiftCodeBlockProps,
};

pub struct SwiftClass<'a> {
    props: SwiftCodeBlockProps<'a>,
}
impl SwiftClass<'_> {
    pub fn get_name(&self) -> Option<String> {
        let x = self.props.node.child_by_field_name("name")?;
        get_node_text(&x, &self.props.text_content)
            .ok()
            .map(|text| String::from_utf16_lossy(&text))
    }
}

impl SwiftCodeBlockBase<'_> for SwiftClass<'_> {
    fn new<'a>(
        tree: &'a SwiftSyntaxTree,
        node: Node<'a>,
        node_metadata: &'a NodeMetadata,
        text_content: &'a XcodeText,
    ) -> Result<SwiftCodeBlock<'a>, SwiftCodeBlockError> {
        Ok(SwiftCodeBlock::Class(SwiftClass {
            props: SwiftCodeBlockProps {
                tree,
                text_content,
                node_metadata,
                node,
            },
        }))
    }

    fn get_kind(&self) -> SwiftCodeBlockKind {
        SwiftCodeBlockKind::Class
    }

    // Boilerplate
    fn as_text(&self) -> Result<XcodeText, SwiftCodeBlockError> {
        get_node_text(&self.props.node, &self.props.text_content)
    }
    fn get_first_char_position(&self) -> TextPosition {
        get_first_char_position(&self.props)
    }
    fn get_last_char_position(&self) -> TextPosition {
        get_last_char_position(&self.props)
    }
    fn get_parent_code_block(&self) -> Result<SwiftCodeBlock, SwiftCodeBlockError> {
        get_parent_code_block(&self.props)
    }
}
