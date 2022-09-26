// Allows calculating a diff on a code document based on a diff of a syntax tree
// The text for changed nodes need to be reconstructed, using nodes in the previous content.
// In principle, any number of nodes may need to have their text range replaced with a new text range.
// The least possible number of nodes should be changed, to avoid writing construction code for every node type

use tree_sitter::Node;

use crate::core_engine::XcodeText;

pub fn construct_text_for_if_statement(conditions: Vec<Node>, child: Node, old_content: XcodeText) {
    return "if "
        + conditions
            .iter()
            .map(|condition| XcodeText::from_array(condition.utf16_text(&old_content)))
            .collect::<Vec<XcodeText>>()
            .join(", ");
}
