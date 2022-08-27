use tree_sitter::Node;
use ts_rs::TS;

use crate::{
    core_engine::{
        features::feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
        rules::{get_bounds_of_TextRange, get_index_of_next_row, MatchRange},
        syntax_tree::SwiftSyntaxTree,
        utils::XcodeText,
        CodeDocument, TextPosition, TextRange,
    },
    utils::geometry::LogicalPosition,
};

use super::{
    models::{BracketHighlightBracketPair, BracketHighlightElbow, BracketHighlightResults},
    utils::{
        get_code_block_parent, get_left_most_column_in_rows,
        get_match_range_of_first_and_last_char_in_node, length_to_code_block_body_start,
        only_whitespace_on_line_until_position, rectanges_of_wrapped_line,
        rectangles_from_match_range,
    },
};
pub struct BracketHighlight<'a> {
    code_document: &'a CodeDocument<'a>,
    compute_results: Option<BracketHighlightComputeResults>,
    visualization_results: Option<BracketHighlightResults>,
}

impl<'a> FeatureBase for BracketHighlight<'a> {
    // TODO: Need to set results to none instead of returning early.
    fn compute(&mut self, trigger: &CoreEngineTrigger) -> Result<(), FeatureError> {
        if !should_compute(trigger) {
            return Ok(());
        }
        let selected_text_range = self
            .code_document
            .selected_text_range()
            .as_ref()
            .ok_or(BracketHighlightError::InsufficientContext)?; // TODO: Return None? Which of these should be finished?

        let text_content = self
            .code_document
            .text_content()
            .as_ref()
            .ok_or(BracketHighlightError::InsufficientContext)?;

        let textarea_ui_element = self
            .code_document
            .textarea_ui_element()
            .as_ref()
            .ok_or(BracketHighlightError::InsufficientContext)?;

        let swift_syntax_tree: &SwiftSyntaxTree =
            self.code_document
                .swift_syntax_tree()
                .as_ref()
                .ok_or(BracketHighlightError::InsufficientContext)?;

        let code_block_node =
            get_code_block_treesitter_node(swift_syntax_tree, selected_text_range, text_content)?;

        let (box_positions, box_match_range) =
            get_start_end_positions_and_range(code_block_node, text_content, selected_text_range);

        let (line_positions, line_match_range) = get_line_start_end_positions_and_range(
            code_block_node,
            &box_match_range,
            &box_positions,
            text_content,
            selected_text_range,
        );

        self.compute_results = Some(BracketHighlightComputeResults {
            box_positions,
            box_match_range,
            line_positions,
            line_match_range,
        });

        return Ok(());
    }

    fn update_visualization(&mut self, trigger: &CoreEngineTrigger) -> Result<(), FeatureError> {
        let text_content = self
            .code_document
            .text_content()
            .as_ref()
            .ok_or(BracketHighlightError::InsufficientContext)?;

        let textarea_ui_element = self
            .code_document
            .textarea_ui_element()
            .as_ref()
            .ok_or(BracketHighlightError::InsufficientContext)?;

        let BracketHighlightComputeResults {
            box_positions,
            box_match_range,
            line_positions,
            line_match_range,
        } = &self
            .compute_results
            .ok_or(BracketHighlightError::UpdatingVisualizationBeforeComputing)?;

        let (first_line_rectangle, last_line_rectangle, first_box_rectangle, last_box_rectangle) = (
            rectangles_from_match_range(&line_match_range.0, &textarea_ui_element),
            rectangles_from_match_range(&line_match_range.1, &textarea_ui_element),
            rectangles_from_match_range(&box_match_range.0, &textarea_ui_element),
            rectangles_from_match_range(&box_match_range.1, &textarea_ui_element),
        );

        let line_pair = BracketHighlightBracketPair::new(
            box_match_range.0.range,
            first_line_rectangle,
            line_positions.0,
            box_match_range.1.range,
            last_line_rectangle,
            line_positions.1,
        );

        let box_pair = BracketHighlightBracketPair::new(
            box_match_range.0.range,
            first_box_rectangle,
            box_positions.0,
            box_match_range.1.range,
            last_box_rectangle,
            box_positions.1,
        );

        // Check if elbow is needed
        let mut elbow_origin = None;
        let mut elbow_origin_x_left_most = false;
        let mut elbow = None;

        // Elbow needed because the open and closing bracket are on different lines
        let is_line_on_same_row = line_positions.0.row == line_positions.1.row;
        if !is_line_on_same_row {
            let first_line_bracket_range = line_match_range.0.range.clone();
            if let Some(next_row_index) =
                get_index_of_next_row(first_line_bracket_range.index, &text_content)
            {
                if let Some(left_most_column) = get_left_most_column_in_rows(
                    TextRange {
                        index: next_row_index,
                        length: line_match_range.1.range.index - next_row_index + 1,
                    },
                    &text_content,
                ) {
                    if let (Some(elbow_match_rectangle), Some(line_pair_last)) = (
                        get_bounds_of_TextRange(
                            &TextRange {
                                index: left_most_column.index,
                                length: 1,
                            },
                            &textarea_ui_element,
                        ),
                        line_pair.last,
                    ) {
                        if line_pair_last.rectangle.origin.x > elbow_match_rectangle.origin.x {
                            // Closing bracket is further to the right than the elbow point
                            elbow_origin = Some(LogicalPosition {
                                x: elbow_match_rectangle.origin.x,
                                y: elbow_match_rectangle.origin.y,
                            });
                        }
                        if let Some(first_line_rectangle) = first_line_rectangle {
                            if first_line_rectangle.origin.x < elbow_match_rectangle.origin.x {
                                // Opening bracket is further to the left than the elbow point
                                elbow_origin = Some(LogicalPosition {
                                    x: first_line_rectangle.origin.x,
                                    y: first_line_rectangle.origin.y,
                                })
                            }
                        }
                    }
                }
            }
        }

        let first_line_wrapped_rectangles =
            rectanges_of_wrapped_line(line_positions.0.row, &text_content, textarea_ui_element);
        if first_line_wrapped_rectangles.len() > 1 {
            if let (
                Some(last_wrapped_line_rectangle),
                Some(first_line_rectangle),
                Some(last_line_rectangle),
            ) = (
                first_line_wrapped_rectangles.last(),
                first_line_rectangle,
                last_line_rectangle,
            ) {
                if last_wrapped_line_rectangle.origin.y != first_line_rectangle.origin.y
                    && last_line_rectangle.origin.y != first_line_rectangle.origin.y
                {
                    // Elbow most to the right because open bracket is not at the end of a wrapped line
                    elbow_origin_x_left_most = true;
                }
            }
        }

        // Check if bottom line should be to the top or bottom of last line rectangle
        let elbow_bottom_line_top = if let Some(elbow_bottom_line_top) =
            only_whitespace_on_line_until_position(
                TextPosition {
                    row: line_positions.1.row,
                    column: if line_positions.1.column == 0 {
                        0
                    } else {
                        line_positions.1.column - 1
                    },
                },
                &text_content,
            ) {
            elbow_bottom_line_top
        } else {
            self.results = None;
            return;
        };

        if elbow_origin_x_left_most {
            elbow = Some(BracketHighlightElbow {
                origin: None,
                bottom_line_top: elbow_bottom_line_top,
                origin_x_left_most: true,
            });
        } else if let Some(elbow_origin) = elbow_origin {
            elbow = Some(BracketHighlightElbow {
                origin: Some(elbow_origin),
                bottom_line_top: elbow_bottom_line_top,
                origin_x_left_most: false,
            });
        }

        todo!(); // Refactor visualization results
        self.visualization_results = Some(BracketHighlightResults {
            lines: line_pair,
            elbow,
            boxes: box_pair,
        });
        Ok(())
    }
}

impl<'a> BracketHighlight<'a> {
    pub fn new(code_document: &'a CodeDocument) -> Self {
        Self {
            compute_results: None,
            code_document,
            visualization_results: None,
        }
    }
}
fn should_compute(trigger: &CoreEngineTrigger) -> bool {
    match trigger {
        CoreEngineTrigger::OnTextContentChange => true,
        CoreEngineTrigger::OnTextSelectionChange => true,
        CoreEngineTrigger::OnVisibleTextRangeChange => todo!(),
        _ => false,
    }
}

fn should_update_visualization(trigger: &CoreEngineTrigger) -> bool {
    match trigger {
        CoreEngineTrigger::OnViewportDimensionsChange => true,
        _ => false,
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum BracketHighlightError {
    #[error("Insufficient context for bracket highlighting")]
    InsufficientContext,
}

struct BracketHighlightComputeResults {
    box_positions: StartAndEndTextPositions,
    box_match_range: MatchRange,
    line_positions: StartAndEndTextPositions,
    line_match_range: MatchRange,
}

/*
    pub fn update_content(&mut self, text_content: &XcodeText) {
        if self.swift_syntax_tree.parse(text_content) {
            self.text_content = Some(text_content.to_owned());
        }
    }

    pub fn update_selected_text_range(&mut self, selected_text_range: &TextRange) {
        self.selected_text_range = Some(selected_text_range.to_owned());
    }
*/

fn get_code_block_treesitter_node<'a>(
    syntax_tree: &'a SwiftSyntaxTree,
    selected_text_range: &'a TextRange,
    text_content: &'a XcodeText,
) -> Result<Option<Node<'a>>, BracketHighlightError> {
    let selected_node = match syntax_tree.get_selected_code_node(&selected_text_range) {
        None => return Ok(None),
        Some(node) => node,
    };

    let code_block_node = match get_code_block_parent(selected_node, false) {
        None => return Ok(None),
        Some(node) => node,
    };

    // TODO: Refactor?
    let length_to_bad_code_block_start =
        length_to_code_block_body_start(&code_block_node, &text_content, selected_text_range.index);

    // If selected block is in bad code block declaration, then get parent
    if length_to_bad_code_block_start.is_some() && length_to_bad_code_block_start.unwrap().1 {
        code_block_node = match get_code_block_parent(code_block_node, true) {
            None => return Ok(None),
            Some(node) => node,
        };
    }
    return Ok(Some(code_block_node));
}

struct StartAndEndTextPositions {
    start: TextPosition,
    end: TextPosition,
}

fn get_start_end_positions_and_range(
    node: &Node,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> (StartAndEndTextPositions, TextRange) {
    let match_range = match get_match_range_of_first_and_last_char_in_node(
        &node,
        &text_content,
        selected_text_range.index,
    ) {
        None => todo!(),
        Some(range) => range,
    };

    let mut box_positions: StartAndEndTextPositions = StartAndEndTextPositions {
        start: TextPosition::from_TSPoint(&node.start_position()),
        end: TextPosition::from_TSPoint(&node.end_position()),
    };
    return (box_positions, match_range);
}

fn get_line_start_end_positions_and_range(
    code_block_node: &Node,
    box_match_range: &TextRange,
    box_positions: &StartAndEndTextPositions,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> (StartAndEndTextPositions, TextRange) {
    let line_brackets_match_range = box_match_range.clone();
    let line_positions = box_positions.clone();

    let is_touching_left_first_char =
        selected_text_range.index == line_brackets_match_range.0.range.index; // TODO

    if is_touching_left_first_char {
        if let Some(parent_node) = code_block_node.parent() {
            if let Some(code_block_parent_node) = get_code_block_parent(parent_node, true) {
                return get_start_end_positions_and_range(
                    &code_block_parent_node,
                    text_content,
                    selected_text_range,
                );
            }
        }
    }
    return (line_positions, line_brackets_match_range);
}
