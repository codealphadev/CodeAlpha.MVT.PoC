use anyhow::anyhow;
use tauri::Manager;
use tracing::debug;
use tree_sitter::Node;

use crate::{
    app_handle,
    core_engine::{
        features::feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
        rules::get_index_of_next_row,
        syntax_tree::SwiftSyntaxTreeError,
        utils::XcodeText,
        CodeDocument, TextPosition, TextRange,
    },
    platform::macos::{
        get_code_document_frame_properties, get_visible_text_range, is_text_of_line_wrapped, GetVia,
    },
    utils::{geometry::LogicalPosition, messaging::ChannelList},
    window_controls::config::AppWindow,
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

use super::{
    models::{BracketHighlightBoxPair, BracketHighlightLines, BracketHighlightResults, Elbow},
    utils::{
        get_char_rectangle_from_text_index, get_code_block_parent,
        get_indexes_of_first_and_last_char_in_node, get_text_index_of_left_most_char_in_range,
        length_to_code_block_body_start, only_whitespace_on_line_until_position,
    },
};

pub struct BracketHighlight {
    compute_results: Option<BracketHighlightComputeResults>,
    visualization_results: Option<BracketHighlightResults>,

    is_activated: bool,
}

impl FeatureBase for BracketHighlight {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated || !self.should_compute(trigger) {
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
            Err(BracketHighlightError::UnterminatedCodeBlock)
            | Err(BracketHighlightError::InsufficientContext) => {
                self.compute_results = None;
                return Ok(());
            }
            Err(err) => {
                self.compute_results = None;
                return Err(err.into());
            }
        };

        let (opening_bracket, closing_bracket) = get_start_end_positions_and_indexes(
            &code_block_node,
            text_content,
            selected_text_range,
        );

        let (line_opening_character, line_closing_character) =
            get_line_start_end_positions_and_indexes(
                &code_block_node,
                opening_bracket,
                closing_bracket,
                text_content,
                selected_text_range,
            );

        self.compute_results = Some(BracketHighlightComputeResults {
            opening_bracket,
            closing_bracket,
            line_opening_char: line_opening_character,
            line_closing_char: line_closing_character,
        });
        Ok(())
    }

    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        if !self.is_activated || !self.should_update_visualization(code_document, trigger)? {
            return Ok(());
        }

        let code_doc_frame_props = get_code_document_frame_properties(&GetVia::Current)
            .map_err(|e| BracketHighlightError::GenericError(e.into()))?;

        let dimensions = code_doc_frame_props.dimensions;
        let text_offset =
            code_doc_frame_props
                .text_offset
                .ok_or(BracketHighlightError::GenericError(anyhow!(
                    "Textoffset None - should never happen"
                )))?;

        self.visualization_results = self
            .calculate_visualization_results(code_document, text_offset + dimensions.origin.x)?
            .map(|res| res.to_local(&dimensions.origin));

        self.publish_visualization(code_document);

        Ok(())
    }

    fn activate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = true;

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = false;

        self.compute_results = None;
        self.visualization_results = None;

        Ok(())
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        self.compute_results = None;
        self.visualization_results = None;
        Ok(())
    }
}

fn get_left_most_char_x(
    text_content: &XcodeText,
    line_opening_char: &PositionAndIndex,
    line_closing_char: &PositionAndIndex,
) -> Result<Option<f64>, BracketHighlightError> {
    if line_opening_char.position.row == line_closing_char.position.row {
        return Ok(Some(line_opening_char.position.column as f64));
    }
    if line_opening_char.position.row > line_closing_char.position.row {
        return Err(BracketHighlightError::GenericError(anyhow!(
            "Opening bracket is after closing bracket"
        )));
    }
    // Elbow needed because the open and closing bracket are on different lines
    let next_row_index = get_index_of_next_row(line_opening_char.index, &text_content).ok_or(
        BracketHighlightError::GenericError(anyhow!(
            "Malformed text content; could not find another row"
        )),
    )?;

    let left_most_char_index = get_text_index_of_left_most_char_in_range(
        TextRange {
            index: next_row_index,
            length: line_closing_char.index - next_row_index + 1,
        },
        &text_content,
    )
    .ok_or(BracketHighlightError::GenericError(anyhow!(
        "Malformed text content; could not find another row"
    )))?;

    match get_char_rectangle_from_text_index(left_most_char_index)? {
        Some(rectangle) => Ok(Some(rectangle.origin.x)),
        None => Ok(None), // Could not compute bounds
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

    fn calculate_visualization_results(
        &self,
        code_document: &CodeDocument,
        text_offset_global: f64,
    ) -> Result<Option<BracketHighlightResults>, BracketHighlightError> {
        let text_content = code_document
            .text_content()
            .as_ref()
            .ok_or(BracketHighlightError::InsufficientContext)?;

        let BracketHighlightComputeResults {
            opening_bracket,
            closing_bracket,
            line_opening_char,
            line_closing_char,
        } = match &self.compute_results.as_ref() {
            Some(compute_results) => compute_results,
            None => {
                return Ok(None);
            }
        };

        let (opening_bracket_rect, closing_bracket_rect) = (
            get_char_rectangle_from_text_index(opening_bracket.index)?,
            get_char_rectangle_from_text_index(closing_bracket.index)?,
        );

        let (line_opening_char_rect, line_closing_char_rect) = (
            get_char_rectangle_from_text_index(line_opening_char.index)?,
            get_char_rectangle_from_text_index(line_closing_char.index)?,
        );

        // Check if the first line is wrapped.
        let mut is_first_line_wrapped = false;
        if opening_bracket_rect.is_some() {
            is_first_line_wrapped =
                is_text_of_line_wrapped(line_opening_char.position.row, &GetVia::Current)
                    .map_err(|err| BracketHighlightError::GenericError(err.into()))?
                    .0;
        }

        // Case: opening and closing bracket are on the same line -> no elbow is computed.
        let is_line_spans_multiple_rows = is_first_line_wrapped
            || (line_opening_char.position.row != line_closing_char.position.row);

        if !is_line_spans_multiple_rows {
            return Ok(Some(BracketHighlightResults {
                lines: BracketHighlightLines {
                    start: line_opening_char_rect.map(|rect| LogicalPosition {
                        x: rect.bottom_right().x,
                        y: rect.bottom_right().y - 1.0,
                    }),
                    end: line_closing_char_rect.map(|rect| LogicalPosition {
                        x: rect.bottom_left().x,
                        y: rect.bottom_left().y - 1.0,
                    }),
                    elbow: None,
                },
                boxes: BracketHighlightBoxPair {
                    opening_bracket: opening_bracket_rect,
                    closing_bracket: closing_bracket_rect,
                },
            }));
        }

        // Check if bottom line should be to the top or bottom of last line rectangle
        let elbow_bottom_line_top = only_whitespace_on_line_until_position(
            TextPosition {
                row: line_closing_char.position.row,
                column: if line_closing_char.position.column == 0 {
                    0
                } else {
                    line_closing_char.position.column - 1
                },
            },
            &text_content,
        )?;

        let line_start_point = line_opening_char_rect.map(|rect| LogicalPosition {
            x: rect.bottom_left().x,
            y: rect.bottom_left().y - 1.0,
        });
        let line_end_point = line_closing_char_rect.map(|rect| {
            if elbow_bottom_line_top {
                LogicalPosition {
                    x: rect.top_left().x,
                    y: rect.top_left().y + 1.0,
                }
            } else {
                LogicalPosition {
                    x: rect.bottom_left().x,
                    y: rect.bottom_left().y,
                }
            }
        });

        // To determine the elbow point we are interested of the left-most text position
        // in the codeblock between opening and closing bracket.
        let mut left_most_char_x =
            match get_left_most_char_x(&text_content, &line_opening_char, &line_closing_char)? {
                None => {
                    // Case: elbow is not within the visible_text_range of Xcode
                    return Ok(Some(BracketHighlightResults {
                        lines: BracketHighlightLines {
                            start: line_start_point,
                            end: line_end_point,
                            elbow: Some(Elbow::EstimatedElbow(text_offset_global)),
                        },
                        boxes: BracketHighlightBoxPair {
                            opening_bracket: opening_bracket_rect,
                            closing_bracket: closing_bracket_rect,
                        },
                    }));
                }
                Some(left_most_char_rect) => left_most_char_rect,
            };

        // Check if maybe opening or closing bracket are further left than the elbow point.
        if let Some(line_opening_char_rect) = line_opening_char_rect {
            if line_opening_char_rect.origin.x < left_most_char_x {
                left_most_char_x = line_opening_char_rect.origin.x;
            }
        }
        if let Some(line_closing_char_rect) = line_closing_char_rect {
            if line_closing_char_rect.origin.x < left_most_char_x {
                left_most_char_x = line_closing_char_rect.origin.x;
            }
        }

        let elbow = if is_first_line_wrapped {
            debug!("Bracket Highlight: rendered wrapped line");
            Some(Elbow::KnownElbow(text_offset_global))
        } else {
            Some(Elbow::KnownElbow(left_most_char_x))
        };

        return Ok(Some(BracketHighlightResults {
            lines: BracketHighlightLines {
                start: line_start_point,
                end: line_end_point,
                elbow,
            },
            boxes: BracketHighlightBoxPair {
                opening_bracket: opening_bracket_rect,
                closing_bracket: closing_bracket_rect,
            },
        }));
    }

    fn publish_visualization(&self, _: &CodeDocument) {
        // TODO: Use proper event syntax
        let _ = app_handle().emit_to(
            &AppWindow::CodeOverlay.to_string(),
            &ChannelList::BracketHighlightResults.to_string(),
            &self.visualization_results,
        );
    }

    fn should_compute(&self, trigger: &CoreEngineTrigger) -> bool {
        match trigger {
            CoreEngineTrigger::OnTextContentChange => false, // The TextSelectionChange is already triggered on text content change
            CoreEngineTrigger::OnTextSelectionChange => true,
            _ => false,
        }
    }

    fn should_update_visualization(
        &self,
        _code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<bool, BracketHighlightError> {
        let compute_results = match self.compute_results.as_ref() {
            Some(compute_results) => compute_results,
            None => return Ok(self.visualization_results.is_some()),
        };

        match trigger {
            CoreEngineTrigger::OnViewportDimensionsChange => Ok(true),
            CoreEngineTrigger::OnVisibleTextRangeChange => {
                let visible_text_range = get_visible_text_range(&GetVia::Current)
                    .map_err(|e| BracketHighlightError::GenericError(e.into()))?;
                if let Some(BracketHighlightResults { lines, boxes }) = self.visualization_results {
                    if boxes.opening_bracket.is_none()
                        && visible_text_range.includes_index(compute_results.opening_bracket.index)
                    {
                        Ok(true)
                    } else if boxes.closing_bracket.is_none()
                        && visible_text_range.includes_index(compute_results.closing_bracket.index)
                    {
                        Ok(true)
                    } else if lines.start.is_none()
                        && visible_text_range
                            .includes_index(compute_results.line_opening_char.index)
                    {
                        Ok(true)
                    } else if lines.end.is_none()
                        && visible_text_range
                            .includes_index(compute_results.line_closing_char.index)
                    {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(true)
                }
            }
            CoreEngineTrigger::OnTextSelectionChange => Ok(true),
            CoreEngineTrigger::OnTextContentChange => Ok(false), // text content change already triggers text selection change
            _ => Ok(false),
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
        Ok(node) => node,
        Err(SwiftSyntaxTreeError::NoTreesitterNodeFound) => return Ok(None),
        Err(err) => return Err(BracketHighlightError::GenericError(err.into())),
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

    if code_block_node.end_position().column == 0 {
        // This (probably) means that the codeblock is non-terminated, and the end is taken as the end of the document.
        return Err(BracketHighlightError::UnterminatedCodeBlock);
    }
    return Ok(Some(code_block_node));
}

#[derive(thiserror::Error, Debug)]
pub enum BracketHighlightError {
    #[error("Insufficient context for bracket highlighting")]
    InsufficientContext,
    #[error("Unsupported codeblock.")]
    UnsupportedCodeblock,
    #[error("Position out of bounds")]
    PositionOutOfBounds,
    #[error("Found unterminated code block")]
    UnterminatedCodeBlock,
    #[error("Something went wrong when executing this BracketHighlighting feature.")]
    GenericError(#[source] anyhow::Error),
}

#[derive(Copy, Clone, Debug)]
struct PositionAndIndex {
    position: TextPosition,
    index: usize,
}

#[derive(Copy, Clone, Debug)]
struct StartAndEndTextPositions {
    start: TextPosition,
    end: TextPosition,
}

#[derive(Copy, Clone, Debug)]
struct BracketHighlightComputeResults {
    opening_bracket: PositionAndIndex,
    closing_bracket: PositionAndIndex,
    line_opening_char: PositionAndIndex,
    line_closing_char: PositionAndIndex,
}

fn get_start_end_positions_and_indexes(
    node: &Node,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> (PositionAndIndex, PositionAndIndex) {
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
        PositionAndIndex {
            position: box_positions.start,
            index: match_ranges.0,
        },
        PositionAndIndex {
            position: box_positions.end,
            index: match_ranges.1,
        },
    );
}

fn get_line_start_end_positions_and_indexes(
    code_block_node: &Node,
    opening_bracket: PositionAndIndex,
    closing_bracket: PositionAndIndex,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> (PositionAndIndex, PositionAndIndex) {
    let is_touching_left_first_char = selected_text_range.index == opening_bracket.index;

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
    return (opening_bracket, closing_bracket);
}
