use serde::{Deserialize, Serialize};

use crate::{
    core_engine::{EditorWindowUid, TextRange},
    platform::macos::{
        get_bounds_for_TextRange, get_textarea_uielement, internal::get_uielement_frame, GetVia,
    },
};

use super::{
    annotations_manager::{
        Annotation, AnnotationError, AnnotationResult, AnnotationShape, AnnotationsManager,
    },
    AnnotationJobInstructions, AnnotationJobTrait, AnnotationKind,
    InstructionBoundsPropertyOfInterest, VisibleTextRangePositioning,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnotationJobSingleChar {
    id: uuid::Uuid,
    char_index: usize,
    kind: AnnotationKind,
    instructions: AnnotationJobInstructions,
    result: Option<AnnotationResult>,
}

impl PartialEq for AnnotationJobSingleChar {
    fn eq(&self, other: &Self) -> bool {
        // Compare all properties except the id
        self.char_index == other.char_index
            && self.kind == other.kind
            && self.instructions == other.instructions
    }
}

impl AnnotationJobTrait for AnnotationJobSingleChar {
    fn new(
        id: uuid::Uuid,
        range: &TextRange,
        kind: AnnotationKind,
        instructions: AnnotationJobInstructions,
    ) -> Self {
        Self {
            id,
            char_index: range.index,
            kind,
            instructions,
            result: None,
        }
    }

    fn id(&self) -> uuid::Uuid {
        self.id
    }

    fn compute_bounds(
        &mut self,
        visible_text_range: &TextRange,
        editor_window_uid: EditorWindowUid,
    ) -> Result<AnnotationResult, AnnotationError> {
        let viewport_positioning = AnnotationsManager::get_visibility_relative_to_viewport(
            self.char_index,
            visible_text_range,
        );

        let mut result = AnnotationResult {
            id: self.id,
            position_relative_to_viewport: viewport_positioning.clone(),
            bounds: None,
        };

        // If the character is not within the VisibleTextRange, we can't compute the bounds
        if viewport_positioning != VisibleTextRangePositioning::Visible {
            self.result = Some(result.clone());
            return Ok(result);
        }

        let textarea_uielement = get_textarea_uielement(&GetVia::Current).unwrap(); // TODO: Extract?, error handling

        let ax_bounds_global = get_bounds_for_TextRange(
            &TextRange {
                index: self.char_index,
                length: 1,
            },
            &GetVia::Hash(editor_window_uid),
        )
        .map_err(|e| AnnotationError::GenericError(e.into()))?;

        // Get current code_doc_frame to convert with sent frame
        let updated_code_doc_origin = get_uielement_frame(&textarea_uielement).unwrap().origin; // TODO: Error handling

        result.bounds = Some(vec![ax_bounds_global.to_local(&updated_code_doc_origin)]);

        self.result = Some(result.clone());
        Ok(result)
    }

    fn compute_bounds_if_missing(
        &mut self,
        visible_text_range: &TextRange,
        editor_window_uid: EditorWindowUid,
    ) -> Result<AnnotationResult, AnnotationError> {
        let viewport_positioning = AnnotationsManager::get_visibility_relative_to_viewport(
            self.char_index,
            visible_text_range,
        );

        if let Some(previous_result) = self.result.as_ref() {
            if let Some(bounds) = previous_result.bounds.as_ref() {
                // Case: there is a previous result and it contains bounds -> we don't need to compute anything
                // and only update the viewport positioning
                let result = AnnotationResult {
                    id: self.id,
                    position_relative_to_viewport: viewport_positioning,
                    bounds: Some(bounds.clone()),
                };

                self.result = Some(result.clone());
                return Ok(result);
            }
        }

        self.compute_bounds(visible_text_range, editor_window_uid)
    }

    fn get_annotation(&self) -> Option<Annotation> {
        if let Some(result) = self.result.as_ref() {
            Some(Self::get_annotation_(self, &result))
        } else {
            return None;
        }
    }
}

impl AnnotationJobSingleChar {
    fn get_annotation_(annotation_job: &Self, result: &AnnotationResult) -> Annotation {
        let mut annotation = Annotation {
            id: result.id,
            kind: annotation_job.kind.clone(),
            char_index: annotation_job.char_index,
            position_relative_to_viewport: result.position_relative_to_viewport.clone(),
            shapes: vec![],
        };

        match annotation_job.instructions.bounds_property_of_interest {
            InstructionBoundsPropertyOfInterest::Frame => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Rectangle(r.clone()))
                        .collect()
                });
            }
            InstructionBoundsPropertyOfInterest::PosTopLeft => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Point(r.top_left()))
                        .collect()
                });
            }
            InstructionBoundsPropertyOfInterest::PosTopRight => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Point(r.top_right()))
                        .collect()
                });
            }
            InstructionBoundsPropertyOfInterest::PosBotLeft => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Point(r.bottom_left()))
                        .collect()
                });
            }
            InstructionBoundsPropertyOfInterest::PosBotRight => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Point(r.bottom_right()))
                        .collect()
                });
            }
        }

        annotation
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core_engine::{
            annotations_manager::{
                annotation_job::InstructionBoundsPropertyOfInterest,
                annotations_manager::{Annotation, AnnotationResult, AnnotationShape},
            },
            TextRange,
        },
        utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    };

    use super::{
        AnnotationJobInstructions, AnnotationJobSingleChar, AnnotationJobTrait, AnnotationKind,
        VisibleTextRangePositioning,
    };

    #[test]
    fn get_annotation_instruct_property_of_interest() {
        let instructions_prop_as_frame = AnnotationJobInstructions::default();
        let instructions_prop_as_top_left_point = AnnotationJobInstructions {
            bounds_property_of_interest: InstructionBoundsPropertyOfInterest::PosTopLeft,
            ..instructions_prop_as_frame
        };

        let annotation_job1 = AnnotationJobSingleChar::new(
            uuid::Uuid::new_v4(),
            &TextRange {
                index: 0,
                length: 1,
            },
            AnnotationKind::CodeblockFirstChar,
            AnnotationJobInstructions::default(),
        );

        let annotation_job2 = AnnotationJobSingleChar::new(
            uuid::Uuid::new_v4(),
            &TextRange {
                index: 0,
                length: 1,
            },
            AnnotationKind::CodeblockFirstChar,
            instructions_prop_as_top_left_point,
        );

        let result = AnnotationResult {
            id: annotation_job1.clone().id,
            position_relative_to_viewport: VisibleTextRangePositioning::Visible,
            bounds: Some(vec![LogicalFrame {
                origin: LogicalPosition { x: 0.0, y: 0.0 },
                size: LogicalSize {
                    width: 10.0,
                    height: 10.0,
                },
            }]),
        };

        let expected_annotation = Annotation {
            id: annotation_job1.clone().id,
            kind: annotation_job1.clone().kind,
            position_relative_to_viewport: VisibleTextRangePositioning::Visible,
            shapes: vec![AnnotationShape::Rectangle(LogicalFrame {
                origin: LogicalPosition { x: 0.0, y: 0.0 },
                size: LogicalSize {
                    width: 10.0,
                    height: 10.0,
                },
            })],
            char_index: 0,
        };

        assert_eq!(
            expected_annotation,
            AnnotationJobSingleChar::get_annotation_(&annotation_job1, &result)
        );

        let expected_annotation = Annotation {
            id: annotation_job1.clone().id,
            kind: annotation_job1.clone().kind,
            position_relative_to_viewport: VisibleTextRangePositioning::Visible,
            shapes: vec![AnnotationShape::Point(LogicalPosition { x: 0.0, y: 0.0 })],
            char_index: 0,
        };

        assert_eq!(
            expected_annotation,
            AnnotationJobSingleChar::get_annotation_(&annotation_job2, &result)
        );
    }
}
