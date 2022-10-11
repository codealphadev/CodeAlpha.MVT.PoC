use anyhow::anyhow;
use tracing::debug;
use tree_sitter::Node;

use crate::{
    core_engine::{
        annotations_manager::{
            AnnotationJob, AnnotationJobInstructions, AnnotationJobSingleChar, AnnotationJobTrait,
            AnnotationKind, InstructionBoundsPropertyOfInterest,
        },
        events::AnnotationManagerEvent,
        features::{
            feature_base::{CoreEngineTrigger, FeatureBase, FeatureError},
            FeatureKind,
        },
        rules::get_index_of_next_row,
        syntax_tree::SwiftSyntaxTreeError,
        utils::XcodeText,
        CodeDocument, EditorWindowUid, TextPosition, TextRange,
    },
    platform::macos::{is_text_of_line_wrapped, GetVia},
    CORE_ENGINE_ACTIVE_AT_STARTUP,
};

use super::utils::{
    get_char_rectangle_from_text_index, get_code_block_parent,
    get_indexes_of_first_and_last_char_in_node, get_text_pos_and_index_of_left_most_char_in_range,
    length_to_code_block_body_start, only_whitespace_on_line_until_position,
};

pub struct BracketHighlight {
    is_activated: bool,

    registered_jobs: Vec<AnnotationJob>,
    group_id: Option<uuid::Uuid>,
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

        let group_id_before_compute = self.group_id;
        if self.compute_procedure(code_document).is_err() {
            if let Some(group_id) = group_id_before_compute {
                AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
                self.group_id = None;
                self.registered_jobs = vec![];
            }
        } else {
            // Annotation group was updated, remove the old one
            if group_id_before_compute == self.group_id {
                if let Some(group_id) = group_id_before_compute {
                    AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
                }
            }
        }

        Ok(())
    }

    fn update_visualization(
        &mut self,
        _code_document: &CodeDocument,
        _trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        Ok(())
    }

    fn activate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = true;

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), FeatureError> {
        self.is_activated = false;

        if let Some(group_id) = self.group_id {
            AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
        }
        self.group_id = None;
        self.registered_jobs = vec![];

        Ok(())
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        if let Some(group_id) = self.group_id {
            AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
        }
        self.group_id = None;
        self.registered_jobs = vec![];
        Ok(())
    }
}

impl BracketHighlight {
    fn compute_procedure(&mut self, code_document: &CodeDocument) -> Result<(), FeatureError> {
        let selected_text_range = match code_document.selected_text_range() {
            Some(range) => range,
            None => {
                if let Some(group_id) = self.group_id {
                    AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
                }
                self.registered_jobs = vec![];
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

        let (opening_bracket, closing_bracket, line_opening_character, line_closing_character) =
            self.compute_annotation_text_positions(
                code_document,
                text_content,
                selected_text_range,
            )?;

        let first_line_is_wrapped = if let Ok((is_wrapped, _)) =
            is_text_of_line_wrapped(line_opening_character.position.row, &GetVia::Current)
        {
            let code_block_spans_multiple_lines = is_wrapped
                || (line_opening_character.position.row != line_closing_character.position.row);

            // Case: opening and closing bracket are on the same line -> no elbow is computed.
            if !code_block_spans_multiple_lines {
                self.register_annotation_jobs(
                    InstructionBoundsPropertyOfInterest::PosBotRight,
                    InstructionBoundsPropertyOfInterest::PosBotLeft,
                    line_closing_character,
                    opening_bracket,
                    closing_bracket,
                    line_opening_character,
                    None,
                    code_document.editor_window_props().window_uid,
                );

                return Ok(());
            }

            Some(is_wrapped)
        } else {
            None
        };

        let elbow_bottom_line_top = only_whitespace_on_line_until_position(
            TextPosition {
                row: line_closing_character.position.row,
                column: if line_closing_character.position.column == 0 {
                    0
                } else {
                    line_closing_character.position.column - 1
                },
            },
            &text_content,
        )?;

        // Check if the first line is wrapped.
        let line_end_corner;
        if elbow_bottom_line_top {
            line_end_corner = InstructionBoundsPropertyOfInterest::PosTopLeft;
        } else {
            line_end_corner = InstructionBoundsPropertyOfInterest::PosBotLeft
        }

        // To determine the elbow point we are interested of the left-most text position
        // in the codeblock between opening and closing bracket.
        let mut left_most_char: PositionAndIndex = match get_elbow_text_position(
            &text_content,
            &line_opening_character,
            &line_closing_character,
        ) {
            Err(_) => {
                let mut elbow = None;
                if first_line_is_wrapped == Some(true) {
                    elbow = Some(PositionAndIndex {
                        position: line_opening_character.position,
                        index: line_opening_character.index
                            - line_opening_character.position.column,
                    });

                    // Check if the highlighted braces within a wrapped row are on the same line.
                    if let (Ok(line_opening_char_rect), Ok(line_closing_char_rect)) = (
                        get_char_rectangle_from_text_index(line_opening_character.index),
                        get_char_rectangle_from_text_index(line_closing_character.index),
                    ) {
                        if let (Some(line_opening_char_rect), Some(line_closing_char_rect)) =
                            (line_opening_char_rect, line_closing_char_rect)
                        {
                            if line_opening_char_rect.origin.y == line_closing_char_rect.origin.y {
                                elbow = Some(line_opening_character)
                            }
                        }
                    }
                }

                self.register_annotation_jobs(
                    InstructionBoundsPropertyOfInterest::PosBotLeft,
                    line_end_corner.clone(),
                    line_closing_character,
                    opening_bracket,
                    closing_bracket,
                    line_opening_character,
                    elbow,
                    code_document.editor_window_props().window_uid,
                );

                return Ok(());
            }
            Ok(left_most_char) => PositionAndIndex {
                position: left_most_char.0,
                index: left_most_char.1,
            },
        };

        let elbow = if first_line_is_wrapped == Some(true) {
            debug!("Bracket Highlight: rendered wrapped line");
            left_most_char.index -= left_most_char.position.column;
            Some(left_most_char)
        } else {
            // Check if maybe opening or closing bracket are further left than the elbow point.
            if line_opening_character.position.column < left_most_char.position.column {
                left_most_char.index -=
                    left_most_char.position.column - line_opening_character.position.column;
            }
            Some(left_most_char)
        };

        self.register_annotation_jobs(
            InstructionBoundsPropertyOfInterest::PosBotLeft,
            line_end_corner,
            line_closing_character,
            opening_bracket,
            closing_bracket,
            line_opening_character,
            elbow,
            code_document.editor_window_props().window_uid,
        );

        Ok(())
    }

    fn compute_annotation_text_positions(
        &mut self,
        code_document: &CodeDocument,
        text_content: &XcodeText,
        selected_text_range: &TextRange,
    ) -> Result<
        (
            PositionAndIndex,
            PositionAndIndex,
            PositionAndIndex,
            PositionAndIndex,
        ),
        FeatureError,
    > {
        let code_block_node = match get_selected_code_block_node(code_document) {
            Ok(node) => {
                if let Some(node) = node {
                    node
                } else {
                    if let Some(group_id) = self.group_id {
                        AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
                    }
                    self.registered_jobs = vec![];
                    return Err(BracketHighlightError::UnsupportedCodeblock.into());
                }
            }
            Err(err) => {
                if let Some(group_id) = self.group_id {
                    AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
                }
                self.registered_jobs = vec![];
                return Err(err.into());
            }
        };

        let (opening_bracket, closing_bracket) = get_start_end_positions_and_indexes(
            &code_block_node,
            text_content,
            selected_text_range,
        )
        .or_else(|e| {
            if let Some(group_id) = self.group_id {
                AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
            }
            self.registered_jobs = vec![];
            Err(e)
        })?;

        let (line_opening_character, line_closing_character) =
            get_line_start_end_positions_and_indexes(
                &code_block_node,
                opening_bracket,
                closing_bracket,
                text_content,
                selected_text_range,
            )
            .or_else(|e| {
                if let Some(group_id) = self.group_id {
                    AnnotationManagerEvent::Remove(group_id).publish_to_tauri();
                }
                self.registered_jobs = vec![];
                Err(e)
            })?;

        Ok((
            opening_bracket,
            closing_bracket,
            line_opening_character,
            line_closing_character,
        ))
    }
}

impl BracketHighlight {
    fn register_annotation_jobs(
        &mut self,
        line_start_corner: InstructionBoundsPropertyOfInterest,
        line_end_corner: InstructionBoundsPropertyOfInterest,
        line_closing_character: PositionAndIndex,
        opening_bracket: PositionAndIndex,
        closing_bracket: PositionAndIndex,
        line_opening_character: PositionAndIndex,
        elbow: Option<PositionAndIndex>,
        window_uid: EditorWindowUid,
    ) {
        let instructions_default = AnnotationJobInstructions::default();
        let instructions_line_start = AnnotationJobInstructions {
            bounds_property_of_interest: line_start_corner,
            ..instructions_default.clone()
        };

        let instructions_line_end = AnnotationJobInstructions {
            bounds_property_of_interest: line_end_corner,
            ..instructions_default.clone()
        };

        let bracket_open = AnnotationJobSingleChar::new(
            uuid::Uuid::new_v4(),
            &TextRange {
                index: opening_bracket.index,
                length: 1,
            },
            AnnotationKind::OpeningBracket,
            instructions_default.clone(),
        );

        let bracket_close = AnnotationJobSingleChar::new(
            uuid::Uuid::new_v4(),
            &TextRange {
                index: closing_bracket.index,
                length: 1,
            },
            AnnotationKind::ClosingBracket,
            instructions_default.clone(),
        );

        let line_start = AnnotationJobSingleChar::new(
            uuid::Uuid::new_v4(),
            &TextRange {
                index: line_opening_character.index,
                length: 1,
            },
            AnnotationKind::LineStart,
            instructions_line_start,
        );

        let line_end = AnnotationJobSingleChar::new(
            uuid::Uuid::new_v4(),
            &TextRange {
                index: line_closing_character.index,
                length: 1,
            },
            AnnotationKind::LineEnd,
            instructions_line_end,
        );

        let mut jobs = vec![
            AnnotationJob::SingleChar(bracket_open),
            AnnotationJob::SingleChar(bracket_close),
            AnnotationJob::SingleChar(line_start),
            AnnotationJob::SingleChar(line_end),
        ];

        if let Some(elbow) = elbow {
            let instructions_elbow = AnnotationJobInstructions {
                bounds_property_of_interest: InstructionBoundsPropertyOfInterest::PosTopLeft,
                ..instructions_default.clone()
            };

            let elbow_job = AnnotationJobSingleChar::new(
                uuid::Uuid::new_v4(),
                &TextRange {
                    index: elbow.index,
                    length: 1,
                },
                AnnotationKind::Elbow,
                instructions_elbow,
            );
            jobs.push(AnnotationJob::SingleChar(elbow_job));
        }

        if self.registered_jobs == jobs && self.group_id.is_some() {
            AnnotationManagerEvent::Update((self.group_id.unwrap(), jobs.clone()))
                .publish_to_tauri();
        } else {
            let new_group_id = uuid::Uuid::new_v4();
            AnnotationManagerEvent::Add((
                new_group_id,
                FeatureKind::BracketHighlight,
                jobs.clone(),
                window_uid,
            ))
            .publish_to_tauri();

            self.group_id = Some(new_group_id);
        }

        self.registered_jobs = jobs;
    }
}

fn get_elbow_text_position(
    text_content: &XcodeText,
    line_opening_char: &PositionAndIndex,
    line_closing_char: &PositionAndIndex,
) -> Result<(TextPosition, usize), BracketHighlightError> {
    if line_opening_char.position.row == line_closing_char.position.row {
        return Err(BracketHighlightError::GenericError(anyhow!(
            "No Elbow needed"
        )));
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

    let left_most_char_index = get_text_pos_and_index_of_left_most_char_in_range(
        TextRange {
            index: next_row_index,
            length: line_closing_char.index - next_row_index + 1,
        },
        &text_content,
    )
    .ok_or(BracketHighlightError::GenericError(anyhow!(
        "Malformed text content; could not find another row"
    )))?;

    Ok(left_most_char_index)
}

impl BracketHighlight {
    pub fn new() -> Self {
        Self {
            registered_jobs: vec![],
            is_activated: CORE_ENGINE_ACTIVE_AT_STARTUP,
            group_id: None,
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
        .get_code_node_by_text_range(selected_text_range)
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

fn get_start_end_positions_and_indexes(
    node: &Node,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> Result<(PositionAndIndex, PositionAndIndex), BracketHighlightError> {
    let mut match_ranges = get_indexes_of_first_and_last_char_in_node(
        &node,
        &text_content,
        selected_text_range.index,
    )?;

    let mut box_positions: StartAndEndTextPositions = StartAndEndTextPositions {
        start: TextPosition::from_TSPoint(&node.start_position()),
        end: TextPosition::from_TSPoint(&node.end_position()),
    };

    if node.kind() == "function_declaration" {
        if let Some(function_declaration_parameters) =
            special_case_function_declaration(node, text_content, selected_text_range)
        {
            match_ranges = (
                function_declaration_parameters.0.index,
                function_declaration_parameters.1.index,
            );
            box_positions = StartAndEndTextPositions {
                start: function_declaration_parameters.0.position,
                end: function_declaration_parameters.1.position,
            };
        } else {
            // Skip function declaration node
            if let Some(parent_node) = node.parent() {
                if let Some(code_block_parent_node) = get_code_block_parent(parent_node, true) {
                    return get_start_end_positions_and_indexes(
                        &code_block_parent_node,
                        text_content,
                        selected_text_range,
                    );
                } else {
                    return Err(BracketHighlightError::UnsupportedCodeblock);
                }
            }
        }
    }

    return Ok((
        PositionAndIndex {
            position: box_positions.start,
            index: match_ranges.0,
        },
        PositionAndIndex {
            position: box_positions.end,
            index: match_ranges.1,
        },
    ));
}

fn special_case_function_declaration(
    node: &Node,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> Option<(PositionAndIndex, PositionAndIndex)> {
    assert!(node.kind() == "function_declaration");

    let mut cursor = node.walk();

    let mut start_position: Option<TextPosition> = None;
    let mut end_position: Option<TextPosition> = None;

    for child in node.children(&mut cursor) {
        if child.kind() == "(" {
            start_position = Some(TextPosition::from_TSPoint(&child.start_position()));
        }

        if child.kind() == ")" {
            end_position = Some(TextPosition::from_TSPoint(&child.start_position()));
        }
    }

    if let (Some(start_position), Some(end_position)) = (start_position, end_position) {
        if let (Some(start_index), Some(end_index)) = (
            start_position.as_TextIndex(&text_content),
            end_position.as_TextIndex(&text_content),
        ) {
            if start_index <= selected_text_range.index && end_index >= selected_text_range.index {
                return Some((
                    PositionAndIndex {
                        position: start_position,
                        index: start_index,
                    },
                    PositionAndIndex {
                        position: end_position,
                        index: end_index,
                    },
                ));
            }
        }
    }

    None
}

fn get_line_start_end_positions_and_indexes(
    code_block_node: &Node,
    opening_bracket: PositionAndIndex,
    closing_bracket: PositionAndIndex,
    text_content: &XcodeText,
    selected_text_range: &TextRange,
) -> Result<(PositionAndIndex, PositionAndIndex), BracketHighlightError> {
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
    return Ok((opening_bracket, closing_bracket));
}
