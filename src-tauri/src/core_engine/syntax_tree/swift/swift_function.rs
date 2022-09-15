use anyhow::anyhow;
use tree_sitter::Node;

use crate::core_engine::{
    syntax_tree::{Complexities, SwiftCodeBlock, SwiftCodeBlockError},
    TextPosition, XcodeText,
};

use super::swift_codeblock::{
    get_first_char_position, get_last_char_position, get_node_text, SwiftCodeBlockBase,
    SwiftCodeBlockKind, SwiftCodeBlockProps,
};

pub struct SwiftFunction<'a> {
    props: SwiftCodeBlockProps<'a>,
}
impl SwiftFunction<'_> {
    pub fn get_complexity(&self) -> isize {
        self.props.node_metadata.get_total_complexity()
    }

    pub fn get_parameter_names(&self) -> Vec<String> {
        let mut cursor = self.props.node.walk();
        let mut result: Vec<String> = Vec::new();
        for node in self
            .props
            .node
            .named_children(&mut cursor)
            .filter(|node| node.kind() == "parameter")
        {
            let internal_name =
                get_internal_name_for_parameter(&node, &self.props.text_content).ok();
            if let Some(internal_name) = internal_name.map(|name| String::from_utf16_lossy(&name)) {
                if internal_name != "_" {
                    result.push(internal_name);
                }
            }
        }
        result
    }
    pub fn get_name(&self) -> Option<String> {
        let x = self.props.node.child_by_field_name("name")?;
        get_node_text(&x, &self.props.text_content)
            .ok()
            .map(|text| String::from_utf16_lossy(&text))
    }
}

fn get_internal_name_for_parameter(
    node: &Node,
    text_content: &XcodeText,
) -> Result<XcodeText, SwiftCodeBlockError> {
    let mut cursor = node.walk();
    for name_node in node.children_by_field_name("name", &mut cursor) {
        if name_node.kind() == "simple_identifier" {
            return get_node_text(&name_node, &text_content);
        }
    }

    return Err(SwiftCodeBlockError::GenericError(anyhow!(
        "get_internal_name_for_parameter: Could not find internal name for parameter: {:?}",
        node
    )));
}

impl SwiftCodeBlockBase<'_> for SwiftFunction<'_> {
    fn new<'a>(
        node: Node<'a>,
        node_metadata: &'a Complexities,
        text_content: &'a XcodeText,
    ) -> Result<SwiftCodeBlock<'a>, SwiftCodeBlockError> {
        Ok(SwiftCodeBlock::Function(SwiftFunction {
            props: SwiftCodeBlockProps {
                text_content,
                node_metadata,
                node,
            },
        }))
    }

    fn get_kind(&self) -> SwiftCodeBlockKind {
        SwiftCodeBlockKind::Function
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
}
