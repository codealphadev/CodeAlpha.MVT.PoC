use anyhow::anyhow;
use std::collections::HashMap;
use tokio::sync::oneshot;
use tracing::error;
use tree_sitter::{Node, Parser, Tree};

use crate::core_engine::{
    syntax_tree::detect_input_edits,
    utils::{TextPosition, TextRange, XcodeText},
};

use super::{calculate_cognitive_complexities, Complexities, SwiftCodeBlockError};

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

type TreeMetaData = HashMap<usize, NodeMetadata>;

#[derive(Clone)]
pub struct SwiftSyntaxTree {
    tree: Tree,
    content: XcodeText,
    node_metadata: TreeMetaData,
}

unsafe impl Send for SwiftSyntaxTree {}
unsafe impl Sync for SwiftSyntaxTree {}

impl SwiftSyntaxTree {
    pub fn new(tree: Tree, node_metadata: TreeMetaData, content: XcodeText) -> Self {
        Self {
            tree,
            content,
            node_metadata,
        }
    }

    pub async fn from_XcodeText(
        content: XcodeText,
        previous_tree: Option<SwiftSyntaxTree>,
    ) -> Result<Self, SwiftSyntaxTreeError> {
        // We wait for a very short time in order to allow quickly subsequently scheduled calls to cancel this one
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        let (send, recv) = oneshot::channel();

        rayon::spawn(move || {
            let tree = Self::parse_content(content, previous_tree);

            _ = send.send(tree);
        });

        match recv.await {
            Ok(ast) => ast,
            Err(e) => Err(SwiftSyntaxTreeError::GenericError(e.into())),
        }
    }

    pub fn _from_XcodeText_blocking(
        content: XcodeText,
        previous_tree: Option<SwiftSyntaxTree>,
    ) -> Result<Self, SwiftSyntaxTreeError> {
        Self::parse_content(content, previous_tree)
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut Tree {
        &mut self.tree
    }

    pub fn text_content(&self) -> &XcodeText {
        &self.content
    }

    pub fn get_metadata_of_node(&self, node: &Node) -> Result<&NodeMetadata, SwiftSyntaxTreeError> {
        self.node_metadata
            .get(&node.id())
            .ok_or(SwiftSyntaxTreeError::NoMetadataFoundForNode)
    }

    pub fn get_code_node_by_text_range(
        &self,
        text_range: &TextRange,
    ) -> Result<Node, SwiftSyntaxTreeError> {
        if let Some((start_position, _)) = text_range.as_StartEndTextPosition(&self.content) {
            if let Some(node) = self.tree.root_node().named_descendant_for_point_range(
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

        Err(SwiftSyntaxTreeError::GenericError(anyhow!(
            "Could not get code node by text range."
        )))
    }

    fn apply_edits_from_diff_to_tree(
        previous_tree: &mut SwiftSyntaxTree,
        new_content: &XcodeText,
    ) -> Result<(), SwiftSyntaxTreeError> {
        const DIFF_DEADLINE_MS: u64 = 20;
        let edits = detect_input_edits(previous_tree.text_content(), new_content, DIFF_DEADLINE_MS);

        for edit in edits {
            previous_tree.tree_mut().edit(&edit);
        }
        return Ok(());
    }

    fn parse_content(
        code_text: XcodeText,
        mut previous_ast: Option<SwiftSyntaxTree>,
    ) -> Result<SwiftSyntaxTree, SwiftSyntaxTreeError> {
        let mut parser = SwiftSyntaxTree::parser();

        let mut previous_TSTree: Option<&Tree> = None;
        if let Some(previous_ast) = &mut previous_ast {
            previous_TSTree = match Self::apply_edits_from_diff_to_tree(previous_ast, &code_text) {
                Ok(_) => Some(previous_ast.tree()),
                Err(_) => None,
            };
        }

        let mut node_metadata: TreeMetaData = HashMap::new();
        match parser.parse_utf16(&code_text, previous_TSTree) {
            Some(tree) => {
                calculate_cognitive_complexities(
                    &tree.root_node(),
                    &code_text,
                    &mut node_metadata,
                    None,
                )?;

                Ok(SwiftSyntaxTree::new(tree, node_metadata, code_text))
            }
            None => Err(SwiftSyntaxTreeError::CouldNotParseTree),
        }
    }

    pub fn parser() -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_swift::language())
            .expect("Swift Language not found");

        parser
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
        let swift_syntax_tree = SwiftSyntaxTree::_from_XcodeText_blocking(text, None).unwrap();
        let root_node = swift_syntax_tree.tree().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(
            root_node.end_byte(),
            swift_syntax_tree.text_content().utf16_bytes_count()
        );
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
        let swift_syntax_tree = SwiftSyntaxTree::_from_XcodeText_blocking(text, None).unwrap();

        let root_node = swift_syntax_tree.tree().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(
            root_node.end_byte(),
            swift_syntax_tree.text_content().utf16_bytes_count()
        );
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
        let mut text = XcodeText::from_str("// 😊\n");
        let mut utf8_str = XcodeText::from_str("let x = 1; cansole.lug(x);");
        text.append(&mut utf8_str);

        let swift_syntax_tree = SwiftSyntaxTree::_from_XcodeText_blocking(text, None).unwrap();

        let root_node = swift_syntax_tree.tree().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(
            root_node.end_byte(),
            swift_syntax_tree.text_content().utf16_bytes_count()
        );
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
