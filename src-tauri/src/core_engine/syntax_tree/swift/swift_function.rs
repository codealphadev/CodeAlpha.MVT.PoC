use anyhow::anyhow;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionParameter {
    pub name: String,
    pub param_type: String,
}

pub struct SwiftFunction<'a> {
    pub props: SwiftCodeBlockProps<'a>,
}
impl SwiftFunction<'_> {
    pub fn get_complexity(&self) -> isize {
        self.props.node_metadata.complexities.get_total_complexity()
    }

    pub fn get_parameters(&self) -> Result<Vec<FunctionParameter>, SwiftCodeBlockError> {
        let mut cursor = self.props.node.walk();
        let mut result: Vec<FunctionParameter> = Vec::new();
        for node in self
            .props
            .node
            .named_children(&mut cursor)
            .filter(|node| node.kind() == "parameter")
        {
            let internal_name =
                get_internal_name_for_parameter(&node, &self.props.text_content).ok();

            let param_type = get_type_for_parameter(&node, &self.props.text_content)?;

            if let Some(internal_name) = internal_name.map(|name| String::from_utf16_lossy(&name)) {
                if internal_name != "_" {
                    result.push(FunctionParameter {
                        name: internal_name,
                        param_type: String::from_utf16_lossy(&param_type),
                    });
                }
            }
        }
        Ok(result)
    }
    pub fn get_name(&self) -> Option<String> {
        let x = self.props.node.child_by_field_name("name")?;
        get_node_text(&x, &self.props.text_content)
            .ok()
            .map(|text| String::from_utf16_lossy(&text))
    }

    pub fn get_top_level_functions<'a>(
        syntax_tree: &'a SwiftSyntaxTree,
        text_content: &'a XcodeText,
    ) -> Result<Vec<SwiftFunction<'a>>, SwiftCodeBlockError> {
        let node = syntax_tree
            .tree()
            .ok_or(SwiftCodeBlockError::GenericError(anyhow!("No tree found")))?
            .root_node();
        Self::get_top_level_functions_recursive(node, syntax_tree, text_content)
    }

    fn get_top_level_functions_recursive<'a>(
        node: Node<'a>,
        syntax_tree: &'a SwiftSyntaxTree,
        text_content: &'a XcodeText,
    ) -> Result<Vec<SwiftFunction<'a>>, SwiftCodeBlockError> {
        let mut results: Vec<SwiftFunction<'a>> = vec![];
        if node.kind() == "function_declaration" {
            let node_metadata = syntax_tree
                .get_node_metadata(&node)
                .map_err(|err| SwiftCodeBlockError::GenericError(err.into()))?;
            let new_func = SwiftFunction::new(syntax_tree, node, node_metadata, &text_content)
                .map(|f| match f {
                    SwiftCodeBlock::Function(f) => f,
                    _ => panic!("Wrong codeblock type"),
                })?;
            results.push(new_func);
        } else {
            for child in node.named_children(&mut node.walk()) {
                results.append(&mut Self::get_top_level_functions_recursive(
                    child,
                    syntax_tree,
                    text_content,
                )?);
            }
        }
        Ok(results)
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

fn get_type_for_parameter(
    node: &Node,
    text_content: &XcodeText,
) -> Result<XcodeText, SwiftCodeBlockError> {
    //https://github.com/alex-pinkus/tree-sitter-swift/blob/f58deb71df91bcee6d650774dbd136a7493ca20f/grammar.js
    const PARAMETER_TYPES: [&str; 10] = [
        "function_type",
        "user_type",
        "tuple_type",
        "array_type",
        "dictionary_type",
        "optional_type",
        "metatype",
        "opaque_type",
        "existential_type",
        "protocol_composition_type",
    ];

    let mut cursor = node.walk();
    for nd in node
        .named_children(&mut cursor)
        .filter(|nd| PARAMETER_TYPES.contains(&nd.kind()))
    {
        return get_node_text(&nd, &text_content);
    }

    return Err(SwiftCodeBlockError::GenericError(anyhow!(
        "get_internal_type_for_parameter: Could not find type for parameter: {:?}",
        node
    )));
}

impl SwiftCodeBlockBase<'_> for SwiftFunction<'_> {
    fn new<'a>(
        tree: &'a SwiftSyntaxTree,
        node: Node<'a>,
        node_metadata: &'a NodeMetadata,
        text_content: &'a XcodeText,
    ) -> Result<SwiftCodeBlock<'a>, SwiftCodeBlockError> {
        Ok(SwiftCodeBlock::Function(SwiftFunction {
            props: SwiftCodeBlockProps {
                tree,
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
    fn get_parent_code_block(&self) -> Result<SwiftCodeBlock, SwiftCodeBlockError> {
        get_parent_code_block(&self.props)
    }
}

#[cfg(test)]
mod tests {
    mod get_top_level_functions {
        use crate::core_engine::{
            syntax_tree::{SwiftFunction, SwiftSyntaxTree},
            XcodeText,
        };

        #[test]
        fn calculates_top_level_functions() {
            let text_content = XcodeText::from_str(
                r#"
                public class LRUAnimationCache: AnimationCacheProvider {
                    public func func1(input: String) {
                    
                    }
                    func func2(input2: String) {
                        func funcInner(input3: String) {
                    
                        }
                    }
                }
            "#,
            );

            let mut swift_syntax_tree = SwiftSyntaxTree::new(SwiftSyntaxTree::parser_mutex());
            swift_syntax_tree.parse(&text_content).unwrap();

            assert_eq!(
                SwiftFunction::get_top_level_functions(&swift_syntax_tree, &text_content)
                    .unwrap()
                    .len(),
                2
            )
        }
    }
}
