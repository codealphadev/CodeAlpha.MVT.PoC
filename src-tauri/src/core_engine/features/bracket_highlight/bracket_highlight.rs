use tauri::Manager;
use tree_sitter::Node;

use crate::{
    core_engine::{
        features::feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
        rules::get_index_of_next_row,
        utils::XcodeText,
        CodeDocument, TextPosition, TextRange,
    },
    utils::{geometry::LogicalPosition, messaging::ChannelList, rule_types::MatchRange},
    window_controls::config::AppWindow,
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
pub struct BracketHighlight {
    compute_results: Option<BracketHighlightComputeResults>,
    visualization_results: Option<BracketHighlightResults>,
}

impl FeatureBase for BracketHighlight {
    // TODO: Need to set results to none instead of returning early.
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !should_compute(trigger) {
            return Ok(());
        }
        let selected_text_range = self
            .code_document
            .selected_text_range()
            .as_ref()
            .ok_or(BracketHighlightError::InsufficientContext)?; // TODO: Return None? Which of these should be finished?

        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    BracketHighlightError::InsufficientContext.into(),
                ))?;

        let selected_text_range = match code_document.selected_text_range() {
            Some(range) => range,
            None => {
                self.compute_results = None;
                todo!("Is this a case"); // TODO: Refactor into separate function?
                return Ok(());
            }
        };

        let code_block_node = match self.get_selected_code_block_node(code_document) {
            Ok(node) => {
                if let Some(node) = node {
                    node
                } else {
                    self.compute_results = None;
                    return Ok(());
                }
            }
            Err(_) => {
                self.compute_results = None;
                return Ok(());
            }
        };

        let (box_positions, box_match_range) =
            get_start_end_positions_and_ranges(&code_block_node, text_content, selected_text_range);

        let (line_positions, line_match_range) = get_line_start_end_positions_and_ranges(
            &code_block_node,
            &box_match_range,
            &box_positions,
            text_content,
            selected_text_range,
        );

        self.compute_results = Some(BracketHighlightComputeResults {
            box_positions,
            box_match_ranges: box_match_range,
            line_positions,
            line_match_ranges: line_match_range,
        });

        return Ok(());
    }

    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    BracketHighlightError::InsufficientContext.into(),
                ))?;

        let textarea_element_hash = code_document.editor_window_props().uielement_hash;

        let BracketHighlightComputeResults {
            box_positions,
            box_match_ranges,
            line_positions,
            line_match_ranges,
        } = &self.compute_results.ok_or(FeatureError::GenericError(
            BracketHighlightError::UpdatingVisualizationBeforeComputing.into(),
        ))?;

        let (first_line_rectangle, last_line_rectangle, first_box_rectangle, last_box_rectangle) = (
            rectangles_from_match_range(&line_match_ranges.start, textarea_element_hash),
            rectangles_from_match_range(&line_match_ranges.end, textarea_element_hash),
            rectangles_from_match_range(&box_match_ranges.start, textarea_element_hash),
            rectangles_from_match_range(&box_match_ranges.end, textarea_element_hash),
        );

        let line_pair = BracketHighlightBracketPair::new(
            box_match_ranges.start.range,
            first_line_rectangle,
            line_positions.start,
            box_match_ranges.end.range,
            last_line_rectangle,
            line_positions.end,
        );

        let box_pair = BracketHighlightBracketPair::new(
            box_match_ranges.start.range,
            first_box_rectangle,
            box_positions.start,
            box_match_ranges.end.range,
            last_box_rectangle,
            box_positions.end,
        );

        // Check if elbow is needed
        let mut elbow_origin = None;
        let mut elbow_origin_x_left_most = false;
        let mut elbow = None;

        // Elbow needed because the open and closing bracket are on different lines
        let is_line_on_same_row = line_positions.start.row == line_positions.end.row;
        if !is_line_on_same_row {
            let first_line_bracket_range = line_match_ranges.start.range.clone();
            if let Some(next_row_index) =
                get_index_of_next_row(first_line_bracket_range.index, &text_content)
            {
                if let Some(left_most_column) = get_left_most_column_in_rows(
                    TextRange {
                        index: next_row_index,
                        length: line_match_ranges.end.range.index - next_row_index + 1,
                    },
                    &text_content,
                ) {
                    if let (Some(elbow_match_rectangle), Some(line_pair_last)) = (
                        get_bounds_of_TextRange(
                            &TextRange {
                                index: left_most_column.index,
                                length: 1,
                            },
                            textarea_element_hash,
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

        let first_line_wrapped_rectangles = rectanges_of_wrapped_line(
            line_positions.start.row,
            &text_content,
            textarea_element_hash,
        );
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
                    row: line_positions.end.row,
                    column: if line_positions.end.column == 0 {
                        0
                    } else {
                        line_positions.end.column - 1
                    },
                },
                &text_content,
            ) {
            elbow_bottom_line_top
        } else {
            self.visualization_results = None;
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

        self.publish_visualization(code_document);
        Ok(())
    }
}

impl BracketHighlight {
    pub fn new() -> Self {
        Self {
            compute_results: None,
            visualization_results: None,
        }
    }

    fn publish_visualization(&self, code_document: &CodeDocument) {
        // TODO: Use proper event syntax
        let _ = code_document.app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            &ChannelList::BracketHighlightResults.to_string(),
            &self.visualization_results,
        );
        todo!();
    }

    // TODO: Move to code document?
    fn get_selected_code_block_node(
        &self,
        code_document: &CodeDocument,
    ) -> Result<Option<Node>, BracketHighlightError> {
        let text_content = code_document
            .text_content()
            .as_ref()
            .ok_or(BracketHighlightError::InsufficientContext)?;

        let selected_text_range = match code_document.selected_text_range() {
            Some(selected_text_range) => selected_text_range,
            None => return Ok(None),
        };

        let selected_node = match code_document
            .syntax_tree()
            .get_selected_code_node(selected_text_range)
        {
            None => return Ok(None),
            Some(node) => node,
        };

        let code_block_node = match get_code_block_parent(selected_node, false) {
            None => return Ok(None),
            Some(node) => node,
        };

        let length_to_bad_code_block_start = length_to_code_block_body_start(
            &code_block_node,
            text_content,
            selected_text_range.index,
        );

        // If selected block is in bad code block declaration, then get parent
        if length_to_bad_code_block_start.is_ok() && length_to_bad_code_block_start.unwrap().1 {
            code_block_node = match get_code_block_parent(code_block_node, true) {
                None => return Ok(None),
                Some(node) => node,
            };
        }
        return Ok(Some(code_block_node));
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

#[derive(thiserror::Error, Debug)]
pub enum BracketHighlightError {
    #[error("Insufficient context for bracket highlighting")]
    InsufficientContext,
    #[error("Attempted to update visualization before computing results")]
    UpdatingVisualizationBeforeComputing,
    #[error("Computing rectangles from match range failed.")]
    ComputingRectFromMatchRangeFailed,
    #[error("Unsupported codeblock.")]
    UnsupportedCodeblock,
    #[error("Something went wrong when executing this BracketHighlighting feature.")]
    GenericError(#[source] anyhow::Error),
}

// TODO: Do we need this?
struct StartAndEndMatchRanges {
    start: MatchRange,
    end: MatchRange,
}

struct BracketHighlightComputeResults {
    box_positions: StartAndEndTextPositions,
    box_match_ranges: StartAndEndMatchRanges,
    line_positions: StartAndEndTextPositions,
    line_match_ranges: StartAndEndMatchRanges,
}

struct StartAndEndTextPositions {
    start: TextPosition,
    end: TextPosition,
}

fn get_start_end_positions_and_ranges(
    node: &Node,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> (StartAndEndTextPositions, StartAndEndMatchRanges) {
    let match_ranges = match get_match_range_of_first_and_last_char_in_node(
        &node,
        &text_content,
        selected_text_range.index,
    ) {
        Ok(range) => range,
        Err(_) => todo!(),
    };

    let mut box_positions: StartAndEndTextPositions = StartAndEndTextPositions {
        start: TextPosition::from_TSPoint(&node.start_position()),
        end: TextPosition::from_TSPoint(&node.end_position()),
    };
    return (
        box_positions,
        StartAndEndMatchRanges {
            start: match_ranges.0,
            end: match_ranges.1,
        },
    );
}

fn get_line_start_end_positions_and_ranges(
    code_block_node: &Node,
    box_match_ranges: &StartAndEndMatchRanges,
    box_positions: &StartAndEndTextPositions,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> (StartAndEndTextPositions, StartAndEndMatchRanges) {
    let line_brackets_match_ranges = *box_match_ranges.clone();
    let line_positions = *box_positions.clone();

    let is_touching_left_first_char =
        selected_text_range.index == line_brackets_match_ranges.start.range.index; // TODO

    if is_touching_left_first_char {
        if let Some(parent_node) = code_block_node.parent() {
            if let Some(code_block_parent_node) = get_code_block_parent(parent_node, true) {
                return get_start_end_positions_and_ranges(
                    &code_block_parent_node,
                    text_content,
                    selected_text_range,
                );
            }
        }
    }
    return (line_positions, line_brackets_match_ranges);
}
