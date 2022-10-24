use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tree_sitter::{Node, Parser, Tree};

use crate::core_engine::utils::{TextPosition, TextRange, XcodeText};

use super::{
    calculate_cognitive_complexities, detect_input_edits, Complexities, SwiftCodeBlockError,
};

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub complexities: Complexities,
}

#[derive(thiserror::Error, Debug)]
pub enum SwiftSyntaxTreeError {
    #[error("No treesitter node could be retreived with the given text range.")]
    NoTreesitterNodeFound,
    #[error("Metadata could not be found for node.")]
    NoMetadataFoundForNode,
    #[error("At this point, no valid tree is available.")]
    NoTreeParsed,
    #[error("Could not parse tree.")]
    CouldNotParseTree,
    #[error("Something went wrong.")]
    GenericError(#[source] anyhow::Error),
}

impl From<SwiftCodeBlockError> for SwiftSyntaxTreeError {
    fn from(error: SwiftCodeBlockError) -> Self {
        SwiftSyntaxTreeError::GenericError(error.into())
    }
}

#[derive(Clone)]
pub struct SwiftSyntaxTree {
    parser: Arc<Mutex<Parser>>,
    tree: Option<Tree>,
    content: Option<XcodeText>,
    node_metadata: HashMap<usize, NodeMetadata>,
}

impl SwiftSyntaxTree {
    pub fn new(parser: Arc<Mutex<Parser>>) -> Self {
        Self {
            parser,
            tree: None,
            content: None,
            node_metadata: HashMap::new(),
        }
    }

    pub fn parser() -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_swift::language())
            .expect("Swift Language not found");

        parser
    }

    pub fn parser_mutex() -> Arc<Mutex<Parser>> {
        Arc::new(Mutex::new(Self::parser()))
    }

    pub fn _reset(&mut self) {
        self.tree = None;
        self.content = None;
        self.node_metadata.clear();
    }

    pub fn get_node_metadata(&self, node: &Node) -> Result<&NodeMetadata, SwiftSyntaxTreeError> {
        self.node_metadata
            .get(&node.id())
            .ok_or(SwiftSyntaxTreeError::NoMetadataFoundForNode)
    }

    pub fn parse(&mut self, content: &XcodeText) -> Result<(), SwiftSyntaxTreeError> {
        if let (Some(old_content), Some(old_tree)) = (&self.content, &mut self.tree) {
            let changes = detect_input_edits(&old_content, content);

            // Changes should be in ascending order of start_byte
            for change in changes.iter().rev() {
                old_tree.edit(change);
            }
        }

        let updated_tree = self.parser.lock().parse_utf16(content, self.tree());

        if let Some(tree) = updated_tree {
            calculate_cognitive_complexities(
                &tree.root_node(),
                &content,
                &mut self.node_metadata,
                None,
            )?;
            self.content = Some(content.to_owned());
            self.tree = Some(tree);
            return Ok(());
        } else {
            return Err(SwiftSyntaxTreeError::CouldNotParseTree);
        }
    }

    pub fn get_code_node_by_text_range(
        &self,
        text_range: &TextRange,
    ) -> Result<Node, SwiftSyntaxTreeError> {
        if let (Some(syntax_tree), Some(text_content)) = (self.tree.as_ref(), self.content.as_ref())
        {
            if let Some((start_position, _)) = text_range.as_StartEndTextPosition(text_content) {
                if let Some(node) = syntax_tree.root_node().named_descendant_for_point_range(
                    TextPosition {
                        row: start_position.row,
                        column: start_position.column,
                    }
                    .as_TSPoint(),
                    TextPosition {
                        row: start_position.row,
                        column: start_position.column,
                    }
                    .as_TSPoint(),
                ) {
                    return Ok(node);
                } else {
                    return Err(SwiftSyntaxTreeError::NoTreesitterNodeFound);
                }
            }
        }

        Err(SwiftSyntaxTreeError::NoTreeParsed)
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }
}

#[cfg(test)]
mod tests_SwiftSyntaxTree {

    use crate::core_engine::utils::{TextPosition, XcodeText};

    use super::SwiftSyntaxTree;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_start_end_point_end_newline_char() {
        let text = XcodeText::from_str("let x = 1; cansole.lug(x);\n");
        //                |------------------------>| <- end column is zero on row 1
        //                                            <- end byte is one past the last byte (27), as they are also zero-based
        let mut swift_syntax_tree = SwiftSyntaxTree::new(SwiftSyntaxTree::parser_mutex());
        swift_syntax_tree.parse(&text).unwrap();

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), text.utf16_bytes_count());
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 1, column: 0 }
        );
    }

    #[test]
    fn test_start_end_point_end_no_newline_char() {
        let text = XcodeText::from_str("let x = 1; cansole.lug(x);");
        //                |------------------------>| <- end column is one past the last char (26)
        //                |------------------------>| <- end byte is one past the last byte (26), as they are also zero-based
        let mut swift_syntax_tree = SwiftSyntaxTree::new(SwiftSyntaxTree::parser_mutex());
        swift_syntax_tree.parse(&text).unwrap();

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), text.utf16_bytes_count());
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 0, column: 26 }
        );
    }

    #[test]
    fn test_start_end_point_with_UTF16_chars() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new(SwiftSyntaxTree::parser_mutex());

        let mut text = XcodeText::from_str("// 😊\n");
        let mut utf8_str = XcodeText::from_str("let x = 1; cansole.lug(x);");
        text.append(&mut utf8_str);

        swift_syntax_tree.parse(&text).unwrap();

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), text.utf16_bytes_count());
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            XcodeText::treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 1, column: 26 }
        );
    }
}
