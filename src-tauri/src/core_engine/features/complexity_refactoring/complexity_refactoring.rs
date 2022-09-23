use tree_sitter;
use tree_sitter::Node;
use tree_sitter_swift::language;

use crate::{
    core_engine::{
        features::{
            complexity_refactoring::check_for_method_extraction,
            feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
        },
        syntax_tree::{SwiftSyntaxTree, SwiftSyntaxTreeError},
        utils::XcodeText,
        CodeDocument, TextRange,
    },
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

pub struct ComplexityRefactoring {
    is_activated: bool,
}
const MAX_ALLOWED_COMPLEXITY: isize = 2; // TODO: Raise to be more reasonable?

impl FeatureBase for ComplexityRefactoring {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated || !self.should_compute(trigger) {
            return Ok(());
        }

        let selected_text_range = match code_document.selected_text_range() {
            Some(selected_text_range) => selected_text_range,
            None => return Ok(()),
        };

        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    ComplexityRefactoringError::InsufficientContext.into(),
                ))?;

        let selected_function = match get_outermost_selected_functionlike(
            selected_text_range,
            code_document.syntax_tree(),
        )? {
            Some(node) => node,
            None => return Ok(()),
        };

        let node_metadata = code_document
            .syntax_tree()
            .get_node_metadata(&selected_function)
            .map_err(|err| ComplexityRefactoringError::GenericError(err.into()))?;

        if node_metadata.complexities.get_total_complexity() <= MAX_ALLOWED_COMPLEXITY {
            println!("This function is fine");
            return Ok(());
        }
        println!("Problem with complexity in this function");
        //check_for_if_combination(&selected_function, text_content);
        check_for_method_extraction(
            selected_function,
            text_content,
            &code_document.syntax_tree(),
        )?;
        Ok(())
    }

    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        Ok(())
    }

    fn activate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = true;

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = false;

        Ok(())
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        Ok(())
    }
}

fn check_for_if_combination(node: &Node, text_content: &XcodeText) {
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let query = tree_sitter::Query::new(
        language(),
        r#"
        (if_statement (statements . (if_statement) @inner-if . )) @outer-if
        "#,
    )
    .unwrap(); // TODO
    let text_string = text_content.as_string();
    let matches = query_cursor.matches(&query, *node, text_string.as_bytes());
    let outer_index = query.capture_index_for_name("outer-if").unwrap();

    for each_match in matches {
        let outer_if_capture = each_match
            .captures
            .iter()
            .filter(|c| c.index == outer_index)
            .last()
            .unwrap();
        let node: Node = outer_if_capture.node;
        dbg!(node.id());
        dbg!(node.child_count());
        dbg!(node.named_child_count());
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            dbg!(child.kind());
        }
    }
}

// TODO: Make a diff of all the node changes to be done, and then only apply text changes for those.

impl ComplexityRefactoring {
    pub fn new() -> Self {
        Self {
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
        }
    }

    fn should_compute(&self, trigger: &CoreEngineTrigger) -> bool {
        match trigger {
            CoreEngineTrigger::OnTextContentChange => false, // The TextSelectionChange is already triggered on text content change
            CoreEngineTrigger::OnTextSelectionChange => true,
            _ => false,
        }
    }
}

fn get_outermost_selected_functionlike<'a>(
    selected_text_range: &'a TextRange,
    syntax_tree: &'a SwiftSyntaxTree,
) -> Result<Option<Node<'a>>, ComplexityRefactoringError> {
    let mut result_node: Option<Node> = None;

    let mut curr_node = match syntax_tree.get_code_node_by_text_range(&selected_text_range) {
        Ok(node) => node,
        Err(SwiftSyntaxTreeError::NoTreesitterNodeFound) => return Ok(None),
        Err(err) => return Err(ComplexityRefactoringError::GenericError(err.into())),
    };

    loop {
        let kind = curr_node.kind();
        if kind == "function_declaration" || kind == "lambda_literal" {
            result_node = Some(curr_node.clone());
        }
        match curr_node.parent() {
            Some(node) => curr_node = node,
            None => break,
        }
    }
    Ok(result_node)
}

#[derive(thiserror::Error, Debug)]
pub enum ComplexityRefactoringError {
    #[error("Insufficient context for complexity refactoring")]
    InsufficientContext,
    #[error("Something went wrong when executing this ComplexityRefactoring feature.")]
    GenericError(#[source] anyhow::Error),
}
