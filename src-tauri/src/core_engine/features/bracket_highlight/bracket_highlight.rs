use tauri::Manager;
use tree_sitter::Node;

use crate::{
    core_engine::{
        features::feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
        rules::get_index_of_next_row,
        utils::XcodeText,
        CodeDocument, TextPosition, TextRange,
    },
    utils::{geometry::LogicalPosition, messaging::ChannelList},
    window_controls::config::AppWindow,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

use super::{
    models::{
        BracketHighlightBoxPair, BracketHighlightElbow, BracketHighlightLines,
        BracketHighlightResults,
    },
    utils::{
        get_char_rectangle_from_text_index, get_code_block_parent,
        get_indexes_of_first_and_last_char_in_node, get_left_most_column_in_rows,
        length_to_code_block_body_start, only_whitespace_on_line_until_position,
        rectanges_of_wrapped_line,
    },
};
pub struct BracketHighlight {
    compute_results: Option<BracketHighlightComputeResults>,
    visualization_results: Option<BracketHighlightResults>,

    is_activated: bool,
}

impl FeatureBase for BracketHighlight {
    // TODO: Need to set results to none instead of returning early.
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated || !should_compute(trigger) {
            return Ok(());
        }

        let selected_text_range = match code_document.selected_text_range() {
            Some(range) => range,
            None => {
                self.compute_results = None;
                return Ok(());
            }
        };

        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    BracketHighlightError::InsufficientContext.into(),
                ))?;

        let code_block_node = match get_selected_code_block_node(code_document) {
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

        let (box_positions, box_match_range) = get_start_end_positions_and_indexes(
            &code_block_node,
            text_content,
            selected_text_range,
        );

        let (line_positions, line_match_range) = get_line_start_end_positions_and_ranges(
            &code_block_node,
            box_match_range,
            box_positions,
            text_content,
            selected_text_range,
        );

        self.compute_results = Some(BracketHighlightComputeResults {
            box_positions,
            box_text_indexes: box_match_range,
            line_positions,
            line_text_indexes: line_match_range,
        });

        return Ok(());
    }

    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.should_update_visualization(trigger) || !self.is_activated {
            return Ok(());
        }

        let text_content =
            code_document
                .text_content()
                .as_ref()
                .ok_or(FeatureError::GenericError(
                    BracketHighlightError::InsufficientContext.into(),
                ))?;

        let window_uid = code_document.editor_window_props().window_uid;

        let BracketHighlightComputeResults {
            box_positions: _,
            box_text_indexes,
            line_positions,
            line_text_indexes,
        } = &self
            .compute_results
            .as_ref()
            .ok_or(FeatureError::GenericError(
                BracketHighlightError::UpdatingVisualizationBeforeComputing.into(),
            ))?;

        let (first_line_rectangle, last_line_rectangle, first_box_rectangle, last_box_rectangle) = (
            get_char_rectangle_from_text_index(line_text_indexes.start, window_uid)?,
            get_char_rectangle_from_text_index(line_text_indexes.end, window_uid)?,
            get_char_rectangle_from_text_index(box_text_indexes.start, window_uid)?,
            get_char_rectangle_from_text_index(box_text_indexes.end, window_uid)?,
        );

        // Check if elbow is needed
        let mut elbow_origin = None;
        let mut elbow_origin_x_left_most = false;

        let is_line_spans_multiple_rows = line_positions.start.row != line_positions.end.row;
        if is_line_spans_multiple_rows {
            // Elbow needed because the open and closing bracket are on different lines
            if let Some(next_row_index) =
                get_index_of_next_row(line_text_indexes.start, &text_content)
            {
                if let Some(left_most_column) = get_left_most_column_in_rows(
                    TextRange {
                        index: next_row_index,
                        length: line_text_indexes.end - next_row_index + 1,
                    },
                    &text_content,
                ) {
                    // Use left-most column in the code block as the elbow origin
                    // TODO: Refactor this logic to use columns etc.
                    if let (Some(elbow_match_rectangle), Some(last_line_rectangle)) = (
                        get_char_rectangle_from_text_index(left_most_column.index, window_uid)?,
                        last_line_rectangle,
                    ) {
                        if last_line_rectangle.origin.x > elbow_match_rectangle.origin.x {
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
            rectanges_of_wrapped_line(line_positions.start.row, &text_content, window_uid);
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
        let elbow_bottom_line_top = only_whitespace_on_line_until_position(
            TextPosition {
                row: line_positions.end.row,
                column: if line_positions.end.column == 0 {
                    0
                } else {
                    line_positions.end.column - 1
                },
            },
            &text_content,
        )?;

        let elbow = if elbow_origin_x_left_most {
            Some(BracketHighlightElbow::LeftMost)
        } else if let Some(elbow_origin) = elbow_origin {
            Some(BracketHighlightElbow::ElbowPoint(elbow_origin))
        } else {
            None
        };

        self.visualization_results = Some(BracketHighlightResults {
            lines: BracketHighlightLines {
                first: first_line_rectangle,
                last: last_line_rectangle,
                elbow,
                bottom_line_top: elbow_bottom_line_top,
            },
            boxes: BracketHighlightBoxPair {
                first: first_box_rectangle,
                last: last_box_rectangle,
            },
        });

        self.publish_visualization(code_document);
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
}

impl BracketHighlight {
    pub fn new() -> Self {
        Self {
            compute_results: None,
            visualization_results: None,
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
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

    fn should_compute(&self, trigger: &CoreEngineTrigger) -> bool {
        match trigger {
            CoreEngineTrigger::OnTextContentChange => true,
            CoreEngineTrigger::OnTextSelectionChange => true,
            _ => false,
        }
    }

    fn should_update_visualization(&self, trigger: &CoreEngineTrigger) -> bool {
        match trigger {
            CoreEngineTrigger::OnViewportDimensionsChange => true,
            // TODO: Make more efficient? Only update visualizations which are missing
            CoreEngineTrigger::OnVisibleTextRangeChange => {
                todo!();
                /*Tif let Some(BracketHighlightResults { lines, boxes }) = self.visualization_results {
                    if lines.first.is_none()
                        || lines.last.is_none()
                        || boxes.first.is_none()
                        || boxes.last.is_none()
                        || lines.elbow.is_none()
                    {
                        return true;
                    }
                } else {*/
                true
                //}
            }
            _ => false,
        }
    }
}

// TODO: Move to code document?
fn get_selected_code_block_node(
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

    let mut code_block_node = match get_code_block_parent(selected_node, false) {
        None => return Ok(None),
        Some(node) => node,
    };

    let length_to_bad_code_block_start =
        length_to_code_block_body_start(&code_block_node, text_content, selected_text_range.index);

    // If selected block is in bad code block declaration, then get parent
    if length_to_bad_code_block_start.is_ok() && length_to_bad_code_block_start.unwrap().1 {
        code_block_node = match get_code_block_parent(code_block_node, true) {
            None => return Ok(None),
            Some(node) => node,
        };
    }
    return Ok(Some(code_block_node));
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
    #[error("Position out of bounds")]
    PositionOutOfBounds,
    #[error("Something went wrong when executing this BracketHighlighting feature.")]
    GenericError(#[source] anyhow::Error),
}

impl From<BracketHighlightError> for FeatureError {
    fn from(cause: BracketHighlightError) -> Self {
        FeatureError::GenericError(cause.into())
    }
}

#[derive(Copy, Clone)]
struct StartAndEndTextIndexes {
    start: usize,
    end: usize,
}

struct BracketHighlightComputeResults {
    box_positions: StartAndEndTextPositions,
    box_text_indexes: StartAndEndTextIndexes,
    line_positions: StartAndEndTextPositions,
    line_text_indexes: StartAndEndTextIndexes,
}

#[derive(Copy, Clone)]
struct StartAndEndTextPositions {
    start: TextPosition,
    end: TextPosition,
}

fn get_start_end_positions_and_indexes(
    node: &Node,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> (StartAndEndTextPositions, StartAndEndTextIndexes) {
    let match_ranges = match get_indexes_of_first_and_last_char_in_node(
        &node,
        &text_content,
        selected_text_range.index,
    ) {
        Ok(range) => range,
        Err(_) => todo!(),
    };

    let box_positions: StartAndEndTextPositions = StartAndEndTextPositions {
        start: TextPosition::from_TSPoint(&node.start_position()),
        end: TextPosition::from_TSPoint(&node.end_position()),
    };
    return (
        box_positions,
        StartAndEndTextIndexes {
            start: match_ranges.0,
            end: match_ranges.1,
        },
    );
}

fn get_line_start_end_positions_and_ranges(
    code_block_node: &Node,
    box_match_ranges: StartAndEndTextIndexes,
    box_positions: StartAndEndTextPositions,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> (StartAndEndTextPositions, StartAndEndTextIndexes) {
    let line_brackets_match_ranges = box_match_ranges;
    let line_positions = box_positions;

    let is_touching_left_first_char = selected_text_range.index == line_brackets_match_ranges.start; // TODO

    if is_touching_left_first_char {
        if let Some(parent_node) = code_block_node.parent() {
            if let Some(code_block_parent_node) = get_code_block_parent(parent_node, true) {
                return get_start_end_positions_and_indexes(
                    &code_block_parent_node,
                    text_content,
                    selected_text_range,
                );
            }
        }
    }
    return (line_positions, line_brackets_match_ranges);
}
